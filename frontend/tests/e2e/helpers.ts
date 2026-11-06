import { expect, type APIRequestContext, type Page } from '@playwright/test';

// Shared helpers for the e2e specs. Not a spec itself: playwright.e2e.config.ts
// only runs files matching *.spec.ts / *.test.ts.

export const BACKEND = 'http://localhost:8000/api/v1';
export const MAILPIT = 'http://localhost:8026';

/** A unique 10-character username that starts with a letter. */
export function uniqueUsername(prefix = 'e2e'): string {
    return (prefix + Math.random().toString(36).slice(2) + Date.now().toString(36)).slice(0, 10);
}

export async function newestMailFor(request: APIRequestContext, email: string): Promise<string> {
    const res = await request.get(`${MAILPIT}/api/v1/messages?limit=50`);
    expect(res.ok(), 'mailpit list API').toBeTruthy();
    const messages: Array<Record<string, unknown>> = (await res.json()).messages ?? [];
    for (const m of messages) {
        if (JSON.stringify(m.To ?? '').includes(email)) {
            const full = await (await request.get(`${MAILPIT}/api/v1/message/${m.ID}`)).json();
            return String(full.Text ?? '') + String(full.HTML ?? '');
        }
    }
    return '';
}

/** Poll Mailpit until the newest mail to `email` matches `pattern`. */
export async function waitForMailMatching(
    request: APIRequestContext,
    email: string,
    pattern: RegExp,
    ms = 20000
): Promise<RegExpMatchArray | null> {
    const deadline = Date.now() + ms;
    while (Date.now() < deadline) {
        const m = (await newestMailFor(request, email)).match(pattern);
        if (m) return m;
        await new Promise((r) => setTimeout(r, 500));
    }
    return null;
}

export type SessionUser = { id: number; username: string; email: string };

/**
 * Register, confirm and log in through the API. `page.request` shares the
 * browser context's cookie jar, so the page is logged in afterwards.
 */
export async function signInViaApi(page: Page, prefix = 'e2e'): Promise<SessionUser> {
    const username = uniqueUsername(prefix);
    const email = `${username}@example.com`;

    const reg = await page.request.post(`${BACKEND}/register`, { data: { username, email } });
    expect(reg.status(), 'register').toBe(200);

    const link = await waitForMailMatching(page.request, email, /confirm-email\/(\S+)/);
    const token = link?.[1].replace(/[.,;:]+$/, '');
    expect(token, 'confirmation token').toBeTruthy();
    expect((await page.request.get(`${BACKEND}/confirm-email/${token}`)).status()).toBe(200);

    const submit = await page.request.post(`${BACKEND}/submit-email`, { data: { email } });
    expect(submit.status(), 'submit-email').toBe(200);
    const otp = (
        await waitForMailMatching(page.request, email, /OTP for your login is ([A-Za-z0-9]{10})/)
    )?.[1];
    expect(otp, 'otp').toHaveLength(10);

    const login = await page.request.post(`${BACKEND}/auth/login`, { data: { email, otp } });
    expect(login.status(), 'login').toBe(200);

    const user = await currentUser(page);
    expect(user, 'logged_in cookie').toBeTruthy();
    return { id: user!.id, username, email };
}

export async function cookie(page: Page, name: string): Promise<string | undefined> {
    return (await page.context().cookies()).find((c) => c.name === name)?.value;
}

/** The user snapshot the backend stores in the `logged_in` cookie. */
export async function currentUser(page: Page): Promise<{ id: number } | null> {
    const raw = await cookie(page, 'logged_in');
    if (!raw) return null;
    try {
        return JSON.parse(decodeURIComponent(raw)).user;
    } catch {
        return JSON.parse(raw).user;
    }
}

export async function xsrfHeaders(page: Page): Promise<Record<string, string>> {
    const xsrf = await cookie(page, 'xsrf_token');
    expect(xsrf, 'xsrf_token cookie').toBeTruthy();
    return { 'X-XSRF-TOKEN': xsrf! };
}

/** Name of some existing tag, or null if the database has none. */
export async function anyTag(request: APIRequestContext): Promise<string | null> {
    const res = await request.post(`${BACKEND}/tags`, { data: { tag: '' } });
    if (!res.ok()) return null;
    const tags: Array<{ name: string }> = (await res.json()).data ?? [];
    return tags[0]?.name ?? null;
}

export async function createQuestion(
    page: Page,
    tag: string,
    title = `Coverage question ${uniqueUsername('q')}?`
): Promise<{ id: string; slug: string; title: string }> {
    const res = await page.request.post(`${BACKEND}/create-question`, {
        data: {
            title,
            description: 'A question body that is comfortably longer than twenty characters.',
            tag_list: [tag]
        },
        headers: await xsrfHeaders(page)
    });
    expect(res.status(), `create-question: ${await res.text()}`).toBe(200);
    const { id, slug } = (await res.json()).data;
    return { id, slug, title };
}

export async function postAnswer(page: Page, questionId: string, value: string) {
    return page.request.post(`${BACKEND}/answer`, {
        data: { id: Number(questionId), value },
        headers: await xsrfHeaders(page)
    });
}

/** A 1x1 transparent PNG. */
export const PNG_1X1 = Buffer.from(
    'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNkYAAAAAYAAjCB0C8AAAAASUVORK5CYII=',
    'base64'
);
