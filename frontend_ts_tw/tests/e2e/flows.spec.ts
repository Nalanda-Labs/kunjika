import { expect, test, type Page } from '@playwright/test';
import { BACKEND, anyTag, createQuestion, postAnswer, signInViaApi, xsrfHeaders } from './helpers';

// Question lifecycle beyond posting: accepting answers (and the karma that
// moves with it), the unanswered list, and editing a question in the UI.

let tag: string;

test.beforeAll(async ({ request }) => {
    const t = await anyTag(request);
    test.skip(!t, 'no tags in the database');
    tag = t!;
});

async function karma(page: Page, uid: number): Promise<number> {
    return (await (await page.request.get(`${BACKEND}/${uid}/summary`)).json()).data.karma;
}

async function answerId(page: Page, qid: string, text: string): Promise<string> {
    const res = await postAnswer(page, qid, text);
    expect(res.status(), await res.text()).toBe(200);
    return (await res.json()).data;
}

test.describe('accepting answers', () => {
    test('only the asker can accept, only answers to that question, and karma follows', async ({
        browser
    }) => {
        const askerCtx = await browser.newContext();
        const asker = await askerCtx.newPage();
        const askerUser = await signInViaApi(asker, 'ask');
        const q = await createQuestion(asker, tag);
        const other = await createQuestion(asker, tag);

        const aCtx = await browser.newContext();
        const a = await aCtx.newPage();
        const aUser = await signInViaApi(a, 'ana');
        const bCtx = await browser.newContext();
        const b = await bCtx.newPage();
        const bUser = await signInViaApi(b, 'anb');

        const ansA = await answerId(a, q.id, 'Answer from A that is long enough.');
        const ansB = await answerId(b, q.id, 'Answer from B that is long enough.');
        const elsewhere = await answerId(b, other.id, 'Answer on another question, long enough.');

        const accept = async (who: Page, qid: string, aid: string) =>
            who.request.post(`${BACKEND}/accept-answer/${qid}/${aid}/`, {
                headers: await xsrfHeaders(who)
            });

        // Not the asker.
        expect((await accept(a, q.id, ansA)).status()).toBe(403);
        // Not an answer to this question.
        expect((await accept(asker, q.id, elsewhere)).status()).toBe(404);
        expect((await accept(asker, q.id, q.id)).status()).toBe(404);

        const karmaA0 = await karma(asker, aUser.id);
        const karmaB0 = await karma(asker, bUser.id);

        // Accept A: A gains karma.
        let r = await accept(asker, q.id, ansA);
        expect(r.status()).toBe(200);
        expect((await r.json()).accepted).toBe(true);
        const karmaA1 = await karma(asker, aUser.id);
        expect(karmaA1).toBeGreaterThan(karmaA0);

        // Move acceptance to B in one click: A loses it, B gains it.
        r = await accept(asker, q.id, ansB);
        expect((await r.json()).accepted).toBe(true);
        expect(await karma(asker, aUser.id)).toBe(karmaA0);
        expect(await karma(asker, bUser.id)).toBe(karmaB0 + (karmaA1 - karmaA0));

        // Clicking the accepted answer again withdraws it.
        r = await accept(asker, q.id, ansB);
        expect((await r.json()).accepted).toBe(false);
        expect(await karma(asker, bUser.id)).toBe(karmaB0);

        // Repeating that does not drain karma.
        await accept(asker, q.id, ansA);
        await accept(asker, q.id, ansB);
        await accept(asker, q.id, ansB);
        expect(await karma(asker, aUser.id)).toBe(karmaA0);
        expect(await karma(asker, bUser.id)).toBe(karmaB0);

        // Accepting your own answer is allowed but earns nothing.
        const own = await answerId(asker, q.id, 'The asker answering their own question.');
        const before = await karma(asker, askerUser.id);
        expect((await accept(asker, q.id, own)).status()).toBe(200);
        expect(await karma(asker, askerUser.id)).toBe(before);

        await Promise.all([askerCtx.close(), aCtx.close(), bCtx.close()]);
    });

    test('the asker accepts an answer from the question page', async ({ browser }) => {
        const askerCtx = await browser.newContext();
        const asker = await askerCtx.newPage();
        await signInViaApi(asker, 'uia');
        const q = await createQuestion(asker, tag);

        const aCtx = await browser.newContext();
        const a = await aCtx.newPage();
        await signInViaApi(a, 'uib');
        const aid = await answerId(a, q.id, 'An answer worth accepting, long enough.');

        await asker.goto(`/questions/${q.id}/${q.slug}`);
        const tick = asker.locator(`#a-${aid}`);
        await expect(tick).toBeVisible({ timeout: 15000 });
        const res = asker.waitForResponse((r) => r.url().includes('/accept-answer/'));
        await tick.click();
        expect((await res).status()).toBe(200);
        await expect(tick).toHaveClass(/answer/);

        await Promise.all([askerCtx.close(), aCtx.close()]);
    });
});

test.describe('unanswered questions', () => {
    test('a new question is unanswered until someone answers it', async ({ browser }) => {
        const askerCtx = await browser.newContext();
        const asker = await askerCtx.newPage();
        await signInViaApi(asker, 'una');
        const q = await createQuestion(asker, tag);

        const list = async () => {
            const res = await asker.request.post(`${BACKEND}/unanswered-questions/`, {
                data: { uat: '', questions_per_page: 30 }
            });
            expect(res.status()).toBe(200);
            const body = await res.json();
            return (body.data.questions as Array<{ id: string }>).map((x) => x.id);
        };
        expect(await list()).toContain(q.id);

        const count = await asker.request.get(`${BACKEND}/get-unanswered-questions-count/`);
        expect((await count.json()).count).toBeGreaterThan(0);

        const aCtx = await browser.newContext();
        const a = await aCtx.newPage();
        await signInViaApi(a, 'unb');
        await answerId(a, q.id, 'Now this question has an answer.');
        expect(await list()).not.toContain(q.id);

        await Promise.all([askerCtx.close(), aCtx.close()]);
    });

    test('the Unanswered tab loads on the questions page', async ({ page }) => {
        await page.goto('/questions');
        const res = page.waitForResponse((r) => r.url().includes('/unanswered-questions/'));
        await page.locator('a[href="#unanswered"]').click();
        expect((await res).status()).toBe(200);
        await expect(page.locator('#unanswered')).toBeVisible();
    });

    test('rejects an oversized page and a bad cursor', async ({ request }) => {
        const big = await request.post(`${BACKEND}/unanswered-questions/`, {
            data: { uat: '', questions_per_page: 100000 }
        });
        expect(big.status()).toBe(400);
        const bad = await request.post(`${BACKEND}/unanswered-questions/`, {
            data: { uat: 'not-a-time', questions_per_page: 10 }
        });
        expect(bad.status()).toBe(400);
    });
});

test.describe('editing a question', () => {
    test('the author edits title and body on the edit page', async ({ page }) => {
        await signInViaApi(page, 'edq');
        const q = await createQuestion(page, tag);

        await page.goto(`/questions/edit/${q.id}`);
        await expect(page.locator('#title')).toHaveValue(q.title, { timeout: 15000 });
        await expect(page.locator('#editNotAuthor')).toHaveCount(0);

        const newTitle = `${q.title} (edited)`;
        await page.locator('#title').fill(newTitle);
        await page
            .locator('textarea, [contenteditable="true"]')
            .first()
            .fill('An edited body that is still comfortably long enough.');
        await page.getByRole('button', { name: 'Edit' }).click();

        await expect(page).toHaveURL(new RegExp(`/questions/${q.id}/`), { timeout: 15000 });
        await expect(page.getByText(newTitle).first()).toBeVisible();
    });

    test("someone else gets a warning instead of the form", async ({ browser }) => {
        const ownerCtx = await browser.newContext();
        const owner = await ownerCtx.newPage();
        await signInViaApi(owner, 'eda');
        const q = await createQuestion(owner, tag);

        const otherCtx = await browser.newContext();
        const other = await otherCtx.newPage();
        await signInViaApi(other, 'edb');
        await other.goto(`/questions/edit/${q.id}`);
        await expect(other.locator('#editNotAuthor')).toBeVisible({ timeout: 15000 });
        await expect(other.getByRole('button', { name: 'Edit' })).toHaveCount(0);

        await Promise.all([ownerCtx.close(), otherCtx.close()]);
    });

    test('anonymous visitors are sent away from the edit page', async ({ page }) => {
        await page.goto('/questions/edit/1');
        await expect(page).toHaveURL(/\/questions$/);
    });

    test('the edit API 404s for a missing post', async ({ page }) => {
        await signInViaApi(page, 'edm');
        const res = await page.request.get(`${BACKEND}/edit/question/999999999`, {
            headers: await xsrfHeaders(page)
        });
        expect(res.status()).toBe(404);
    });
});

test('regular users see no Pin control and cannot pin through the API', async ({ page }) => {
    await signInViaApi(page, 'pin');
    const q = await createQuestion(page, tag);

    await page.goto(`/questions/${q.id}/${q.slug}`);
    // The author's own controls render (Edit), but Pin is for moderators.
    await expect(page.locator('a[title="Edit this post"]').first()).toBeVisible({
        timeout: 15000
    });
    await expect(page.locator('a[title="Pin this post"]')).toHaveCount(0);

    const res = await page.request.post(`${BACKEND}/bookmark/${q.id}`, {
        headers: await xsrfHeaders(page)
    });
    expect(res.status()).toBe(403);
});
