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

// Deleting posts, notifications, followed tags, edit history, search paging
// and the sitemap, through the UI. The moderator-only screens (closing
// questions, suspending users, the report queue) are in moderation.spec.ts.

let tag: string;

test.beforeAll(async ({ request }) => {
    const t = await anyTag(request);
    test.skip(!t, 'no tags in the database');
    tag = t!;
});

test('an author deletes their own answer from the question page', async ({ page }) => {
    await signInViaApi(page, 'del');
    const q = await createQuestion(page, tag);
    const keep = `An answer that stays ${uniqueUsername('k')} on the page.`;
    const gone = `An answer that is deleted ${uniqueUsername('g')} by its author.`;
    expect((await postAnswer(page, q.id, keep)).status()).toBe(200);
    const res = await postAnswer(page, q.id, gone);
    const goneId = (await res.json()).data as string;

    await page.goto(`/questions/${q.id}/${q.slug}`);
    await expect(page.getByText(gone)).toBeVisible({ timeout: 15000 });

    // With answers present, the author is not offered Delete on the question.
    await expect(page.locator(`a[href="/delete/${q.id}"]`)).toHaveCount(0);

    page.once('dialog', (d) => d.accept());
    await page.locator(`a[href="/delete/${goneId}"]`).click();
    await expect(page.getByText(gone)).toHaveCount(0);
    await expect(page.getByText(keep)).toBeVisible();

    // It is gone for good, not just hidden.
    await page.reload();
    await expect(page.getByText(keep)).toBeVisible({ timeout: 15000 });
    await expect(page.getByText(gone)).toHaveCount(0);
});

test('an author deletes their unanswered question', async ({ page }) => {
    await signInViaApi(page, 'delq');
    const q = await createQuestion(page, tag);
    await page.goto(`/questions/${q.id}/${q.slug}`);
    await expect(page.getByText(q.title).first()).toBeVisible({ timeout: 15000 });

    page.once('dialog', (d) => d.accept());
    await page.locator(`a[href="/delete/${q.id}"]`).click();
    await expect(page).toHaveURL(/\/questions$/, { timeout: 15000 });
    const again = await page.request.get(`${BACKEND}/questions/${q.id}/${q.slug}`);
    expect(again.status()).not.toBe(200);
});

test('an answer shows up as a notification for the asker', async ({ browser }) => {
    const askerCtx = await browser.newContext();
    const asker = await askerCtx.newPage();
    await signInViaApi(asker, 'ntfa');
    const q = await createQuestion(asker, tag);

    const helperCtx = await browser.newContext();
    const helper = await helperCtx.newPage();
    const helperUser = await signInViaApi(helper, 'ntfh');
    const res = await postAnswer(helper, q.id, 'A helpful answer that is long enough to be accepted.');
    const answerId = (await res.json()).data as string;

    // The bell shows one unread, and the page says who did what.
    await asker.goto('/questions');
    await expect(asker.locator('#unreadBadge')).toHaveText('1', { timeout: 15000 });
    await asker.getByRole('link', { name: /Notifications/ }).click();
    await expect(asker).toHaveURL(/\/notifications$/);
    const row = asker.getByRole('list', { name: 'Notifications' }).getByRole('listitem').first();
    await expect(row).toContainText(`${helperUser.username} answered`);
    await expect(row.getByRole('link', { name: q.title })).toHaveAttribute(
        'href',
        `/questions/${q.id}/${q.slug}#${answerId}`
    );

    await asker.getByRole('button', { name: 'Mark all as read' }).click();
    await expect(asker.locator('#unreadBadge')).toHaveCount(0);
    await expect(asker.getByRole('button', { name: 'Mark all as read' })).toHaveCount(0);

    // The email setting is on by default and is saved when changed.
    const email = asker.getByRole('checkbox', { name: /Email me/ });
    await expect(email).toBeChecked();
    const saved = asker.waitForResponse((r) => r.url().includes('/notifications/settings/') && r.request().method() === 'POST');
    await email.uncheck();
    await saved;
    await asker.reload();
    await expect(asker.getByRole('checkbox', { name: /Email me/ })).not.toBeChecked({ timeout: 15000 });

    // Anonymous visitors are sent to log in instead.
    await helperCtx.close();
    await askerCtx.close();
    const anon = await (await browser.newContext()).newPage();
    await anon.goto('/notifications');
    await expect(anon.getByText(/log in/i).first()).toBeVisible();
});

test('following a tag from its page', async ({ page }) => {
    await signInViaApi(page, 'fol');
    await page.goto(`/questions/tagged/${encodeURIComponent(tag)}`);
    const follow = page.getByRole('button', { name: /^Follow(ing)?$/ });
    await expect(follow).toHaveText('Follow', { timeout: 15000 });
    await follow.click();
    await expect(follow).toHaveText('Following');

    // It is remembered, and listed with the notification settings.
    await page.reload();
    await expect(page.getByRole('button', { name: 'Following' })).toBeVisible({ timeout: 15000 });
    await page.goto('/notifications');
    await expect(page.getByRole('link', { name: tag, exact: true })).toBeVisible({ timeout: 15000 });
    await page.getByRole('button', { name: `Unfollow ${tag}` }).click();
    await expect(page.getByText(/not following any tags/)).toBeVisible();

    // The tag page advertises its RSS feed.
    await page.goto(`/questions/tagged/${encodeURIComponent(tag)}`);
    const feed = await page.getByRole('link', { name: 'RSS' }).getAttribute('href');
    expect(feed).toContain(`/feeds/tagged/${encodeURIComponent(tag)}/`);
    const xml = await page.request.get(feed!);
    expect(xml.status()).toBe(200);
    expect(await xml.text()).toContain('<rss');
});

test('an edited answer links to its history', async ({ page }) => {
    await signInViaApi(page, 'his');
    const q = await createQuestion(page, tag);
    const original = `The original wording ${uniqueUsername('o')} of the answer.`;
    const edited = `The corrected wording ${uniqueUsername('e')} of the answer.`;
    const res = await postAnswer(page, q.id, original);
    const answerId = (await res.json()).data as string;

    // Never edited: no History link.
    await page.goto(`/questions/${q.id}/${q.slug}`);
    await expect(page.getByText(original)).toBeVisible({ timeout: 15000 });
    await expect(page.locator(`a[href="/posts/${answerId}/history"]`)).toHaveCount(0);

    const edit = await page.request.post(`${BACKEND}/edit/answer/${answerId}`, {
        data: { description: edited },
        headers: await xsrfHeaders(page)
    });
    expect(edit.status()).toBe(200);

    await page.reload();
    await expect(page.getByText(edited)).toBeVisible({ timeout: 15000 });
    await page.locator(`a[href="/posts/${answerId}/history"]`).click();
    await expect(page.getByRole('heading', { name: 'Edit history' })).toBeVisible();
    await expect(page.getByLabel('Current version')).toContainText(edited);
    await expect(page.getByLabel('Earlier version')).toContainText(original);
});

test('search results are paged', async ({ page }) => {
    test.setTimeout(180_000);
    // 21 matching questions: one more than a page. Three askers, because
    // questions are rate limited to 10 an hour each.
    const word = `pagedsearch${uniqueUsername('w')}`.replace(/[^a-z0-9]/gi, '');
    for (let asker = 0; asker < 3; asker++) {
        await signInViaApi(page, `srch${asker}`);
        for (let n = 0; n < 7; n++) {
            await createQuestion(page, tag, `Searching for ${word} attempt ${asker}-${n}?`);
        }
    }

    await page.goto(`/search?q=${word}`);
    await expect(page.getByRole('heading', { name: new RegExp(`\\(21\\)`) })).toBeVisible({ timeout: 15000 });
    await expect(page.getByText('Page 1 of 2')).toBeVisible();
    await expect(page.getByRole('link', { name: new RegExp(`Searching for ${word}`) })).toHaveCount(20);

    await page.getByRole('button', { name: 'Next page' }).click();
    await expect(page).toHaveURL(/page=2/);
    await expect(page.getByText('Page 2 of 2')).toBeVisible();
    await expect(page.getByRole('link', { name: new RegExp(`Searching for ${word}`) })).toHaveCount(1);
});

test('robots.txt points at a sitemap that lists questions', async ({ page, request }) => {
    await signInViaApi(page, 'map');
    const q = await createQuestion(page, tag);

    const robots = await request.get('/robots.txt');
    expect(robots.status()).toBe(200);
    expect(await robots.text()).toMatch(/^Sitemap: https?:\/\/\S+\/sitemap\.xml$/m);

    const sitemap = await request.get('/sitemap.xml');
    expect(sitemap.status()).toBe(200);
    expect(sitemap.headers()['content-type']).toContain('xml');
    expect(await sitemap.text()).toContain(`/questions/${q.id}/${q.slug}</loc>`);
});
