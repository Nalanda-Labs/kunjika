import { expect, test } from '@playwright/test';
import {
    BACKEND,
    anyTag,
    createQuestion,
    postAnswer,
    signInViaApi,
    uniqueUsername,
    xsrfHeaders
} from './helpers';

// Asking, answering, editing, voting, search and profiles, end to end.

let tag: string;

test.beforeAll(async ({ request }) => {
    const t = await anyTag(request);
    test.skip(!t, 'no tags in the database');
    tag = t!;
});

test('ask a question through the UI and land on its page', async ({ page }) => {
    await signInViaApi(page, 'ask');
    const title = `How do I test coverage ${uniqueUsername('t')}?`;

    await page.goto('/ask');
    await expect(page.locator('#title')).toBeVisible({ timeout: 15000 });
    await page.locator('#title').fill(title);
    await page
        .locator('textarea, [contenteditable="true"]')
        .first()
        .fill('This description is definitely long enough to pass validation.');
    const tags = page.locator('#tags');
    await tags.click();
    await tags.fill(tag);
    await tags.press('Tab');

    await page.getByRole('button', { name: /^\s*Ask\s*$/ }).click();

    await expect(page).toHaveURL(/\/questions\/\d+\//, { timeout: 15000 });
    await expect(page.getByText(title).first()).toBeVisible();
});

test('the API rejects a question that is too short', async ({ page }) => {
    await signInViaApi(page, 'ask');
    const res = await page.request.post(`${BACKEND}/create-question`, {
        data: { title: 'Hi', description: 'short', tag_list: [tag] },
        headers: await xsrfHeaders(page)
    });
    expect(res.status()).toBe(400);
});

test('answers: posted, listed, and too-short ones rejected', async ({ page, request }) => {
    await signInViaApi(page, 'ans');
    const q = await createQuestion(page, tag);

    const answerText = `An answer long enough to be accepted ${uniqueUsername('a')}`;
    const ok = await postAnswer(page, q.id, answerText);
    expect(ok.status(), await ok.text()).toBe(200);

    const short = await postAnswer(page, q.id, 'too short');
    expect(short.status(), 'answer under 20 chars').toBe(400);

    // Answers are paged with limit/offset and come with the total count.
    const list = await request.post(`${BACKEND}/question/get-answers/${q.id}/?limit=10&offset=0`);
    expect(list.status()).toBe(200);
    const listed = await list.json();
    expect(JSON.stringify(listed)).toContain(answerText);
    expect(listed.count).toBe(1);

    // And the question page shows it.
    await page.goto(`/questions/${q.id}/${q.slug}`);
    await expect(page.getByText(answerText)).toBeVisible({ timeout: 15000 });
});

test('answers are paginated: page links, ?page= and #answer deep links', async ({ page }) => {
    test.setTimeout(120_000);
    await signInViaApi(page, 'pag');
    const q = await createQuestion(page, tag);

    // 21 answers: one more than a page of 20.
    const ids: string[] = [];
    for (let n = 1; n <= 21; n++) {
        const res = await postAnswer(page, q.id, `Paginated answer number ${n} of twenty-one.`);
        expect(res.status(), await res.text()).toBe(200);
        ids.push((await res.json()).data as string);
    }
    const first = page.getByText('Paginated answer number 1 of twenty-one.');
    const last = page.getByText('Paginated answer number 21 of twenty-one.');

    // Page one holds the first 20; the 21st is on page two.
    await page.goto(`/questions/${q.id}/${q.slug}`);
    await expect(first).toBeVisible({ timeout: 15000 });
    await expect(page.getByText('Page 1 of 2')).toBeVisible();
    await expect(last).toHaveCount(0);

    await page.getByRole('button', { name: 'Next page' }).click();
    await expect(page).toHaveURL(/\?page=2$/);
    await expect(last).toBeVisible();
    await expect(first).toHaveCount(0);
    await expect(page.getByText('Page 2 of 2')).toBeVisible();

    // Back in the browser history returns to page one.
    await page.goBack();
    await expect(first).toBeVisible();
    await expect(page.getByText('Page 1 of 2')).toBeVisible();

    // A link to one answer opens the page that answer is on, and ?page= works directly.
    await page.goto('/');
    await page.goto(`/questions/${q.id}/${q.slug}#${ids[20]}`);
    await expect(last).toBeVisible({ timeout: 15000 });
    await expect(page.getByText('Page 2 of 2')).toBeVisible();
    await page.goto('/');
    await page.goto(`/questions/${q.id}/${q.slug}?page=2`);
    await expect(last).toBeVisible({ timeout: 15000 });

    // A new answer posted from page one belongs on page two: the page follows it.
    await page.goto(`/questions/${q.id}/${q.slug}`);
    await expect(first).toBeVisible({ timeout: 15000 });
    await page.getByRole('link', { name: 'Reply' }).last().click();
    const fresh = `A twenty-second answer ${uniqueUsername('p')} posted from the UI.`;
    await page.locator('#editor textarea, #editor [contenteditable="true"]').first().fill(fresh);
    await page.getByRole('button', { name: 'Reply' }).click();
    await expect(page).toHaveURL(/\?page=2#\d+$/);
    await expect(page.getByText(fresh)).toBeVisible();
    await expect(last).toBeVisible();
});

test('authors can edit their answer in the UI; others cannot', async ({ browser }) => {
    const authorCtx = await browser.newContext();
    const author = await authorCtx.newPage();
    await signInViaApi(author, 'own');
    const q = await createQuestion(author, tag);
    const res = await postAnswer(author, q.id, 'Original answer text that is long enough.');
    const answerId = (await res.json()).data as string;

    // Author: the edit form is offered and saves.
    await author.goto(`/answers/edit/${answerId}`);
    await expect(author.getByRole('heading', { name: /Edit your answer/i })).toBeVisible();
    await expect(author.locator('#editNotAuthor')).toHaveCount(0);
    const edit = await author.request.post(`${BACKEND}/edit/answer/${answerId}`, {
        data: { description: 'Edited answer text that is also long enough.' },
        headers: await xsrfHeaders(author)
    });
    expect(edit.status()).toBe(200);

    // Someone else: warned in the UI and refused by the API.
    const otherCtx = await browser.newContext();
    const other = await otherCtx.newPage();
    await signInViaApi(other, 'oth');
    await other.goto(`/answers/edit/${answerId}`);
    await expect(other.locator('#editNotAuthor')).toBeVisible({ timeout: 15000 });
    const hijack = await other.request.post(`${BACKEND}/edit/answer/${answerId}`, {
        data: { description: 'Hijacked answer text that is long enough.' },
        headers: await xsrfHeaders(other)
    });
    expect(hijack.status()).toBe(403);
    const hijackQ = await other.request.post(`${BACKEND}/edit-post/${q.id}`, {
        data: {
            title: 'Hijacked title',
            description: 'Hijacked body that is long enough.',
            tag_list: [tag]
        },
        headers: await xsrfHeaders(other)
    });
    expect(hijackQ.status()).toBe(403);

    await authorCtx.close();
    await otherCtx.close();
});

test('voting: no self-votes, one vote per user, score moves', async ({ browser }) => {
    const aCtx = await browser.newContext();
    const a = await aCtx.newPage();
    await signInViaApi(a, 'va');
    const q = await createQuestion(a, tag);

    const self = await a.request.post(`${BACKEND}/votes`, {
        data: { id: Number(q.id), vote: 1 },
        headers: await xsrfHeaders(a)
    });
    expect((await self.json()).success).toBe(false);

    const bCtx = await browser.newContext();
    const b = await bCtx.newPage();
    await signInViaApi(b, 'vb');
    const headers = await xsrfHeaders(b);
    const first = await b.request.post(`${BACKEND}/votes`, {
        data: { id: Number(q.id), vote: 1 },
        headers
    });
    expect(first.status(), await first.text()).toBe(200);
    const again = await b.request.post(`${BACKEND}/votes`, {
        data: { id: Number(q.id), vote: 1 },
        headers
    });
    expect((await again.json()).success, 'second identical vote refused').toBe(false);

    const shown = await b.request.get(`${BACKEND}/questions/${q.id}/${q.slug}`);
    const data = (await shown.json()).data;
    expect(data.votes).toBe(1);
    expect(data.vote_by_current_user, 'voter id comes from the session').toBe(1);

    await aCtx.close();
    await bCtx.close();
});

test('search page finds a freshly asked question', async ({ page }) => {
    await signInViaApi(page, 'srch');
    const word = `zq${uniqueUsername('').replace(/[^a-z]/g, '')}x`;
    const q = await createQuestion(page, tag, `Searchable ${word} question title`);

    await page.goto(`/search?q=${word}`);
    await expect(page.getByRole('heading', { name: new RegExp(`Search results for "${word}"`) }))
        .toBeVisible();
    await expect(page.locator(`a[href*="/questions/${q.id}/"]`).first()).toBeVisible({
        timeout: 15000
    });
});

test('search page says when nothing matches', async ({ page }) => {
    await page.goto(`/search?q=nomatch${Date.now()}zzz`);
    await expect(page.getByText(/No questions matched your search/i)).toBeVisible({
        timeout: 15000
    });
});

test('profile page shows the user and their summary', async ({ page }) => {
    const user = await signInViaApi(page, 'prof');
    await page.goto(`/users/${user.id}/${user.username}`);
    await expect(page.getByText(user.username).first()).toBeVisible({ timeout: 15000 });

    const summary = await page.request.get(`${BACKEND}/${user.id}/summary`);
    expect(summary.status()).toBe(200);
    expect((await summary.json()).success).toBe(true);
});

test('bookmarks are private to their owner', async ({ browser }) => {
    const aCtx = await browser.newContext();
    const a = await aCtx.newPage();
    const userA = await signInViaApi(a, 'bka');

    const bCtx = await browser.newContext();
    const b = await bCtx.newPage();
    await signInViaApi(b, 'bkb');
    const peek = await b.request.post(`${BACKEND}/${userA.id}/get-bookmarks-by-user/`, {
        data: { uat: '', bookmarks_per_page: 10 },
        headers: await xsrfHeaders(b)
    });
    expect(peek.status()).toBe(400);

    await aCtx.close();
    await bCtx.close();
});

test('markdown help page renders', async ({ page }) => {
    await page.goto('/markdown-help');
    await expect(page.getByRole('heading', { name: /Introduction/i })).toBeVisible();
    await expect(page.getByRole('heading', { name: /Math Support/i })).toBeVisible();
});
