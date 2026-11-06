import { expect, test } from '@playwright/test';
import {
    BACKEND,
    PNG_1X1,
    cookie,
    signInViaApi,
    uniqueUsername,
    waitForMailMatching,
    xsrfHeaders
} from './helpers';

// Account lifecycle: profile editing, avatar, user search, logout, deleting
// the account, and the email-link pages.

test.describe('profile settings', () => {
    test('edit the profile through the Settings tab', async ({ page }) => {
        const user = await signInViaApi(page, 'pro');
        const dialogs: string[] = [];
        page.on('dialog', (d) => {
            dialogs.push(d.message());
            d.accept();
        });

        await page.goto(`/users/${user.id}/${user.username}`);
        await page.locator('a[href="#settings"]').click();
        await expect(page.locator('#designation')).toBeVisible({ timeout: 15000 });

        await page.locator('#designation').fill('Test Engineer');
        await page.locator('#location').fill('Testville');
        const saved = page.waitForResponse((r) => r.url().endsWith('/update-profile'));
        await page.getByRole('button', { name: /save|update|submit/i }).first().click();
        expect((await saved).status()).toBe(200);
        await expect.poll(() => dialogs).toContain('Profile data saved');

        const profile = await (
            await page.request.get(`${BACKEND}/user/${user.id}/${user.username}`)
        ).json();
        expect(profile.data.designation).toBe('Test Engineer');
        expect(profile.data.location).toBe('Testville');
    });

    test('profile updates are validated', async ({ page, browser }) => {
        const user = await signInViaApi(page, 'pv');
        const base = { designation: '', location: '', git: '', website: '' };
        const update = (data: Record<string, string>) =>
            xsrfHeaders(page).then((headers) =>
                page.request.post(`${BACKEND}/${user.id}/update-profile`, {
                    data: { ...base, ...data },
                    headers
                })
            );

        for (const username of ['ab', '1abc', 'a@b.c', 'x'.repeat(34)]) {
            const res = await update({ username });
            expect(res.status(), `username "${username}"`).toBe(400);
        }
        expect((await update({ username: user.username, designation: 'd'.repeat(65) })).status())
            .toBe(400);

        // Someone else's name is a conflict, not a server error.
        const otherCtx = await browser.newContext();
        const other = await signInViaApi(await otherCtx.newPage(), 'pw');
        const taken = await update({ username: other.username });
        expect(taken.status()).toBe(409);
        await otherCtx.close();

        // A valid change goes through.
        const renamed = uniqueUsername('new');
        expect((await update({ username: renamed })).status()).toBe(200);
    });

    test("you cannot update or read someone else's private profile data", async ({
        page,
        browser
    }) => {
        const otherCtx = await browser.newContext();
        const victim = await signInViaApi(await otherCtx.newPage(), 'vic');
        await signInViaApi(page, 'att');
        const headers = await xsrfHeaders(page);

        const upd = await page.request.post(`${BACKEND}/${victim.id}/update-profile`, {
            data: { username: 'hijacked', designation: '', location: '', git: '', website: '' },
            headers
        });
        expect(upd.status()).toBe(401);

        const img = await page.request.post(`${BACKEND}/${victim.id}/profile-image-upload`, {
            multipart: { file: { name: 'a.png', mimeType: 'image/png', buffer: PNG_1X1 } },
            headers
        });
        expect(img.status()).toBe(401);

        const del = await page.request.post(`${BACKEND}/delete-profile/${victim.id}`, { headers });
        expect(del.status()).toBe(401);

        const priv = await page.request.get(`${BACKEND}/user/${victim.id}`, { headers });
        expect(priv.status()).toBe(403);
        await otherCtx.close();
    });

    test('upload a profile image', async ({ page }) => {
        const user = await signInViaApi(page, 'avt');
        const headers = await xsrfHeaders(page);

        const bad = await page.request.post(`${BACKEND}/${user.id}/profile-image-upload`, {
            multipart: { file: { name: 'x.png', mimeType: 'image/png', buffer: Buffer.from('nope') } },
            headers
        });
        expect(bad.status()).toBe(400);

        const ok = await page.request.post(`${BACKEND}/${user.id}/profile-image-upload`, {
            multipart: { file: { name: 'a.png', mimeType: 'image/png', buffer: PNG_1X1 } },
            headers
        });
        expect(ok.status(), await ok.text()).toBe(200);
        const { url } = await ok.json();

        const profile = await (
            await page.request.get(`${BACKEND}/user/${user.id}/${user.username}`)
        ).json();
        expect(profile.data.image_url).toBe(url);

        await page.goto(`/users/${user.id}/${user.username}`);
        await expect(page.locator(`img[src="${url}"]`).first()).toBeVisible({ timeout: 15000 });
    });
});

test.describe('users directory', () => {
    test('search users from the users page', async ({ page, browser }) => {
        const ctx = await browser.newContext();
        const target = await signInViaApi(await ctx.newPage(), 'fnd');
        await ctx.close();

        await page.goto('/users');
        const search = page.getByPlaceholder('Filter by user');
        await search.fill(target.username);
        const res = page.waitForResponse((r) => r.url().endsWith('/search-users'));
        await search.press('Enter');
        expect((await res).status()).toBe(200);
        await expect(page.getByText(target.username).first()).toBeVisible({ timeout: 15000 });
    });

    test('search-users caps the page size', async ({ request }) => {
        for (const users_per_page of [-1, 0, 100000]) {
            const res = await request.post(`${BACKEND}/search-users`, {
                data: { username: '', users_per_page }
            });
            expect(res.status(), `users_per_page=${users_per_page}`).toBe(200);
            expect(((await res.json()).data as unknown[]).length).toBeLessThanOrEqual(30);
        }
    });

    test('user activity tabs list their questions and answers', async ({ page }) => {
        const user = await signInViaApi(page, 'act');
        const headers = await xsrfHeaders(page);
        const tags = await (await page.request.post(`${BACKEND}/tags`, { data: { tag: '' } })).json();
        test.skip(!tags.data?.length, 'no tags in the database');
        const q = await page.request.post(`${BACKEND}/create-question`, {
            data: {
                title: `Activity tab question ${uniqueUsername('a')}`,
                description: 'Body long enough for the twenty character rule.',
                tag_list: [tags.data[0].name]
            },
            headers
        });
        const { id: qid } = (await q.json()).data;
        await page.request.post(`${BACKEND}/answer`, {
            data: { id: Number(qid), value: 'My own answer that is long enough.' },
            headers
        });

        const qs = await page.request.post(`${BACKEND}/${user.id}/get-questions-by-user/`, {
            data: { uat: '' }
        });
        expect(qs.status()).toBe(200);
        expect((await qs.json()).count).toBe(1);

        const as = await page.request.post(`${BACKEND}/${user.id}/get-answers-by-user/`, {
            data: { uat: '' }
        });
        expect(as.status()).toBe(200);
        expect((await as.json()).count).toBe(1);

        await page.goto(`/users/${user.id}/${user.username}`);
        const qTab = page.waitForResponse((r) => r.url().includes('/get-questions-by-user/'));
        await page.locator('a[href="#questions"]').click();
        expect((await qTab).status()).toBe(200);
        await expect(page.getByText(/Activity tab question/).first()).toBeVisible();

        const aTab = page.waitForResponse((r) => r.url().includes('/get-answers-by-user/'));
        await page.locator('a[href="#answers"]').click();
        expect((await aTab).status()).toBe(200);
    });
});

test.describe('signing out and deleting the account', () => {
    test('Sign Out logs out in the browser and revokes the session', async ({
        page,
        playwright
    }) => {
        await signInViaApi(page, 'out');

        // Loading a page refreshes the session (rotating the tokens), so take
        // the tokens that are current when Sign Out is clicked.
        const refreshed = page.waitForResponse((r) => r.url().includes('/auth/refresh'));
        await page.goto('/questions');
        await refreshed;
        const access = await cookie(page, 'access_token');
        const xsrf = await cookie(page, 'xsrf_token');

        await page.getByRole('button', { name: 'Sign Out' }).click();
        await expect(page).toHaveURL(/\/questions/);
        await expect(page.getByRole('button', { name: 'Sign Out' })).toHaveCount(0);

        // The old token no longer authenticates anywhere.
        const fresh = await playwright.request.newContext();
        const res = await fresh.post(`${BACKEND}/reports/open-count/`, {
            headers: { 'X-XSRF-TOKEN': xsrf!, Cookie: `access_token=${access}` }
        });
        expect(res.status()).toBe(401);
        await fresh.dispose();
    });

    test('logout API needs a session', async ({ request }) => {
        const res = await request.post(`${BACKEND}/auth/logout`);
        expect(res.status()).toBeGreaterThanOrEqual(400);
        expect(res.status()).toBeLessThan(500);
    });

    test('delete your account from the profile page', async ({ page, request }) => {
        const user = await signInViaApi(page, 'del');
        page.on('dialog', (d) => d.accept());

        await page.goto(`/users/${user.id}/${user.username}`);
        await page.locator('a[href="#settings"]').click();
        await page.locator('a[href="#delete-profile"]').click();
        await page.getByRole('button', { name: 'Delete Profile' }).click();
        await expect(page).toHaveURL(/\/questions/, { timeout: 15000 });

        // The account can no longer receive a login code.
        const otp = await request.post(`${BACKEND}/submit-email`, { data: { email: user.email } });
        expect(otp.status()).toBe(401);

        // And its old session is dead.
        const after = await page.request.post(`${BACKEND}/reports/open-count/`, {
            headers: await xsrfHeaders(page).catch(() => ({}))
        });
        expect(after.status()).toBeGreaterThanOrEqual(400);
    });
});

test.describe('email link pages', () => {
    test('confirm-email page confirms a fresh registration', async ({ page, request }) => {
        const username = uniqueUsername('cfm');
        const email = `${username}@example.com`;
        expect(
            (await request.post(`${BACKEND}/register`, { data: { username, email } })).status()
        ).toBe(200);
        const token = (
            await waitForMailMatching(request, email, /confirm-email\/(\S+)/)
        )?.[1].replace(/[.,;:]+$/, '');

        await page.goto(`/confirm-email/${token}`);
        await expect(page.getByText(/Email verified/i)).toBeVisible({ timeout: 15000 });
    });

    test('resend confirmation email sends a new link', async ({ request }) => {
        const username = uniqueUsername('rsd');
        const email = `${username}@example.com`;
        await request.post(`${BACKEND}/register`, { data: { username, email } });
        const first = (await waitForMailMatching(request, email, /confirm-email\/(\S+)/))?.[1]
            .replace(/[.,;:]+$/, '');

        const res = await request.get(`${BACKEND}/resend-confirmation-email/${first}`);
        expect(res.status()).toBe(200);
        expect((await res.json()).success).toBe(true);

        const bad = await request.get(`${BACKEND}/resend-confirmation-email/forged-token`);
        expect(bad.status()).toBe(400);
    });
});
