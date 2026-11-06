import { expect, test } from '@playwright/test';
import { BACKEND, PNG_1X1, cookie, signInViaApi, uniqueUsername, xsrfHeaders } from './helpers';

// Security and robustness behaviour of the backend as seen from the browser:
// uploads, session handling, malformed input and account-enumeration checks.

test.describe('image uploads', () => {
    test('anonymous upload is rejected', async ({ request }) => {
        const res = await request.post(`${BACKEND}/image-upload`, {
            multipart: { file: { name: 'a.png', mimeType: 'image/png', buffer: PNG_1X1 } }
        });
        expect(res.status(), 'no session, no upload').toBeGreaterThanOrEqual(400);
        expect(res.status()).toBeLessThan(500);
    });

    test('a non-image file is rejected even when logged in', async ({ page }) => {
        await signInViaApi(page, 'upl');
        const res = await page.request.post(`${BACKEND}/image-upload`, {
            multipart: {
                file: {
                    name: 'evil.png',
                    mimeType: 'image/png',
                    buffer: Buffer.from('<html><script>alert(1)</script></html>')
                }
            },
            headers: await xsrfHeaders(page)
        });
        expect(res.status()).toBe(400);
        expect((await res.json()).message).toMatch(/Only PNG, JPEG, GIF and WebP/);
    });

    test('a PNG is stored and served back as a sandboxed image', async ({ page }) => {
        await signInViaApi(page, 'upl');
        const res = await page.request.post(`${BACKEND}/image-upload`, {
            multipart: { file: { name: 'dot.png', mimeType: 'image/png', buffer: PNG_1X1 } },
            headers: await xsrfHeaders(page)
        });
        expect(res.status(), await res.text()).toBe(200);
        const { url } = await res.json();
        expect(url).toMatch(/\/uploads\/\d+\/[A-Za-z]+\/\d+\/[0-9a-f-]{36}$/);

        const img = await page.request.get(url);
        expect(img.status()).toBe(200);
        expect(img.headers()['content-type']).toBe('image/png');
        expect(img.headers()['x-content-type-options']).toBe('nosniff');
        expect(img.headers()['content-security-policy']).toContain("default-src 'none'");
        expect(Buffer.compare(await img.body(), PNG_1X1)).toBe(0);
    });

    test('upload paths cannot escape the upload folder', async ({ request }) => {
        for (const path of [
            'uploads/2026/November/6/..%2F..%2Ftemplate.test.json',
            'uploads/..%2F..%2F/November/6/0f8fad5b-d9cb-469f-a165-70867728950e',
            'uploads/2026/November/6/not-a-uuid'
        ]) {
            const res = await request.get(`${BACKEND}/${path}`);
            expect(res.status(), path).toBe(404);
        }
    });
});

test.describe('sessions', () => {
    test('refresh rotates tokens: the old ones work briefly, then stop', async ({
        page,
        playwright
    }) => {
        // Waits out the backend's ROTATION_GRACE_SECS (10s).
        test.slow();
        await signInViaApi(page, 'rot');
        const oldRefresh = await cookie(page, 'refresh_token');
        const oldAccess = await cookie(page, 'access_token');
        const oldXsrf = await cookie(page, 'xsrf_token');

        const refreshed = await page.request.get(`${BACKEND}/auth/refresh`);
        expect(refreshed.status()).toBe(200);
        expect(await cookie(page, 'refresh_token'), 'new refresh token issued').not.toBe(
            oldRefresh
        );

        // The new session works...
        const ok = await page.request.post(`${BACKEND}/reports/open-count/`, {
            headers: await xsrfHeaders(page)
        });
        expect(ok.status(), 'new access token accepted').not.toBe(401);

        // A fresh context carries only the cookies we give it, so nothing from
        // the live session leaks in.
        const fresh = await playwright.request.newContext();
        const withOldAccess = () =>
            fresh.post(`${BACKEND}/reports/open-count/`, {
                headers: { 'X-XSRF-TOKEN': oldXsrf!, Cookie: `access_token=${oldAccess}` }
            });

        // Requests already in flight with the old token still succeed...
        expect((await withOldAccess()).status(), 'old token within grace').not.toBe(401);

        // ...but once the grace period is over the old token is dead.
        await page.waitForTimeout(11_000);
        expect((await withOldAccess()).status(), 'old access token retired').toBe(401);

        // Replaying the spent refresh token logs the client out instead of
        // minting another session.
        const replay = await fresh.get(`${BACKEND}/auth/refresh`, {
            headers: { Cookie: `refresh_token=${oldRefresh}` }
        });
        expect(replay.status()).toBe(200);
        expect(await replay.json()).toEqual({ status: true });
        expect(replay.headers()['set-cookie'] ?? '').toMatch(/access_token=;/);
        await fresh.dispose();
    });

    test('a malformed logged_in cookie renders pages as logged out', async ({ page, context }) => {
        await context.addCookies([
            { name: 'logged_in', value: 'not-json', domain: 'localhost', path: '/' }
        ]);
        const errors: number[] = [];
        page.on('response', (r) => {
            if (r.url().startsWith('http://localhost:5173') && r.status() >= 500)
                errors.push(r.status());
        });
        await page.goto('/questions');
        await expect(page.getByRole('heading', { name: /questions/i }).first()).toBeVisible();
        expect(errors, 'no 5xx from the SvelteKit server').toHaveLength(0);
    });

    test('a forged logged_in cookie does not make the backend treat you as that user', async ({
        request
    }) => {
        // The backend used to read the viewer's id from this unsigned cookie.
        const forged = JSON.stringify({ user: { id: 1, username: 'admin' } });
        const res = await request.post(`${BACKEND}/questions/`, {
            data: { uat: '', questions_per_page: 1 }
        });
        const first = (await res.json()).data?.questions?.[0];
        test.skip(!first, 'no questions in the database');

        const q = await request.get(`${BACKEND}/questions/${first.id}/${first.slug}`, {
            headers: { Cookie: `logged_in=${encodeURIComponent(forged)}` }
        });
        expect(q.status()).toBe(200);
        // An anonymous viewer has never voted.
        expect((await q.json()).data.vote_by_current_user).toBe(0);
    });
});

test.describe('malformed input gets 4xx, not a crash', () => {
    test('bad pagination cursor', async ({ request }) => {
        const res = await request.post(`${BACKEND}/questions/`, {
            data: { uat: 'yesterday-ish', questions_per_page: 5 }
        });
        expect(res.status()).toBe(400);
        // The backend is still serving afterwards.
        const again = await request.post(`${BACKEND}/questions/`, {
            data: { uat: '', questions_per_page: 5 }
        });
        expect(again.status()).toBe(200);
    });

    test('non-numeric ids', async ({ request }) => {
        expect((await request.get(`${BACKEND}/questions/abc/slug`)).status()).toBe(400);
        expect((await request.get(`${BACKEND}/user/abc/someone`)).status()).toBe(400);
        expect((await request.get(`${BACKEND}/abc/summary`)).status()).toBe(400);
    });

    test('database errors are not echoed to clients', async ({ request }) => {
        const res = await request.get(`${BACKEND}/get-tag-info/${uniqueUsername('nosuch')}`);
        expect(res.status()).toBe(404);
        expect(JSON.stringify(await res.json())).not.toMatch(/sqlx|SQLSTATE|relation|column/i);
    });
});

test.describe('email links and login codes', () => {
    test('confirm-email page explains an invalid link', async ({ page }) => {
        await page.goto('/confirm-email/not-a-real-token');
        await expect(page.getByText(/Invalid signature/i)).toBeVisible();
    });

    test('a login code works only once', async ({ page, request }) => {
        // signInViaApi consumed the code; replaying it must fail.
        const user = await signInViaApi(page, 'otp');
        const mails = await request.get(`http://localhost:8026/api/v1/messages?limit=50`);
        expect(mails.ok()).toBeTruthy();
        const msg = ((await mails.json()).messages as Array<Record<string, unknown>>).find((m) =>
            JSON.stringify(m.To).includes(user.email)
        );
        const body = await (
            await request.get(`http://localhost:8026/api/v1/message/${msg!.ID}`)
        ).json();
        const otp = String(body.Text).match(/OTP for your login is ([A-Za-z0-9]{10})/)?.[1];
        expect(otp).toHaveLength(10);

        const replay = await request.post(`${BACKEND}/auth/login`, {
            data: { email: user.email, otp }
        });
        expect(replay.status()).toBe(401);
    });
});
