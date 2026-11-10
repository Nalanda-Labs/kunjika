import { expect, test, type Page } from '@playwright/test';
import {
    BACKEND,
    anyTag,
    createQuestion,
    moderatorUnavailable,
    postAnswer,
    signInAsModerator,
    signInViaApi,
    xsrfHeaders
} from './helpers';

// The moderator-only screens: the report queue, closing and reopening
// questions, deleting other people's posts, and suspending users.
//
// A moderator can only be made in the database (see `signInAsModerator`), so
// these are skipped when the test database is not reachable with psql.

let tag: string;

test.beforeAll(async ({ request }) => {
    const why = moderatorUnavailable();
    test.skip(!!why, why ?? '');
    const t = await anyTag(request);
    test.skip(!t, 'no tags in the database');
    tag = t!;
});

async function report(page: Page, postId: string) {
    const res = await page.request.post(`${BACKEND}/report-post/`, {
        data: { post_id: Number(postId), reason: 'spam', details: 'e2e moderation' },
        headers: await xsrfHeaders(page)
    });
    expect(res.status(), await res.text()).toBe(200);
}

test('a moderator sees the Reports link and works the queue', async ({ browser }) => {
    const authorCtx = await browser.newContext();
    const author = await authorCtx.newPage();
    await signInViaApi(author, 'rpa');
    const reporterCtx = await browser.newContext();
    const reporter = await reporterCtx.newPage();
    await signInViaApi(reporter, 'rpr');

    // The queue shows 20 at a time; make sure there is more than one page.
    // Reports are limited to 5 a day per user and questions to 10 an hour, so
    // top the queue up only as far as needed, with fresh users as required.
    const mod = await (await browser.newContext()).newPage();
    await signInAsModerator(mod, 'rpm');
    const openCount = async () =>
        (await (await mod.request.post(`${BACKEND}/reports/open-count/`, { headers: await xsrfHeaders(mod) })).json())
            .count as number;
    const dismissed = await createQuestion(author, tag);
    await report(reporter, dismissed.id);
    let filed = 1;
    while ((await openCount()) < 21) {
        if (filed % 5 === 0) {
            await signInViaApi(reporter, 'rpr');
            await signInViaApi(author, 'rpa');
        }
        await report(reporter, (await createQuestion(author, tag)).id);
        filed++;
    }
    const total = await openCount();

    // Ordinary users have no Reports link; the moderator's has the open count.
    await author.goto('/questions');
    await expect(author.getByRole('link', { name: /Reports/ })).toHaveCount(0);
    await mod.goto('/questions');
    const link = mod.getByRole('link', { name: /Reports/ });
    await expect(link).toContainText(String(total), { timeout: 15000 });
    await link.click();
    await expect(mod).toHaveURL(/\/reports$/);

    // First page, then the rest through the cursor.
    const cards = mod.locator('.card');
    await expect(cards).toHaveCount(20, { timeout: 15000 });
    await expect(mod.getByText(`${total} open reports`)).toBeVisible();
    const more = mod.getByRole('button', { name: 'Load more reports' });
    await more.click();
    await expect(cards).toHaveCount(Math.min(40, total));
    while (await more.count()) {
        const before = await cards.count();
        await more.click();
        await expect.poll(() => cards.count()).toBeGreaterThan(before);
    }
    await expect(cards).toHaveCount(total);

    // Dismissing a report takes it out of the list without reloading it.
    const card = cards.filter({ has: mod.getByRole('link', { name: dismissed.title }) });
    await expect(card).toHaveCount(1);
    mod.once('dialog', (d) => d.accept());
    await card.getByRole('button', { name: 'Dismiss' }).click();
    await expect(card).toHaveCount(0);
    await expect(cards).toHaveCount(total - 1);
    await expect(mod.getByText(`${total - 1} open reports`)).toBeVisible();

    // It is filed under "dismissed", and the question is still there.
    await mod.getByRole('button', { name: 'dismissed' }).click();
    await expect(mod.getByRole('link', { name: dismissed.title }).first()).toBeVisible({ timeout: 15000 });
    expect((await mod.request.get(`${BACKEND}/questions/${dismissed.id}/${dismissed.slug}`)).status()).toBe(200);
});

test('"Delete post" on a report removes the reported question', async ({ browser }) => {
    const author = await (await browser.newContext()).newPage();
    await signInViaApi(author, 'rda');
    const reporter = await (await browser.newContext()).newPage();
    await signInViaApi(reporter, 'rdr');
    const q = await createQuestion(author, tag);
    await report(reporter, q.id);

    const mod = await (await browser.newContext()).newPage();
    await signInAsModerator(mod, 'rdm');
    await mod.goto('/reports');
    // Newest first, so the fresh report is on the first page.
    const card = mod.locator('.card').filter({ has: mod.getByRole('link', { name: q.title }) });
    await expect(card).toHaveCount(1, { timeout: 15000 });
    mod.once('dialog', (d) => {
        expect(d.message()).toContain('DELETE');
        d.accept();
    });
    await card.getByRole('button', { name: 'Delete post' }).click();
    await expect(card).toHaveCount(0);
    expect((await mod.request.get(`${BACKEND}/questions/${q.id}/${q.slug}`)).status()).not.toBe(200);
});

test('a moderator closes a question as a duplicate and reopens it', async ({ browser }) => {
    const user = await (await browser.newContext()).newPage();
    await signInViaApi(user, 'clu');
    const original = await createQuestion(user, tag);
    const dup = await createQuestion(user, tag);

    const mod = await (await browser.newContext()).newPage();
    await signInAsModerator(mod, 'clm');
    await mod.goto(`/questions/${dup.id}/${dup.slug}`);
    await expect(mod.getByText(dup.title).first()).toBeVisible({ timeout: 15000 });

    // Paste the original's link. Its port number must not be read as the id.
    mod.once('dialog', (d) => d.accept(`http://localhost:5173/questions/${original.id}/${original.slug}`));
    await mod.getByRole('link', { name: 'Close' }).click();
    const notice = mod.locator('#closedNotice');
    await expect(notice).toContainText('closed as a duplicate of');
    await expect(notice.getByRole('link', { name: original.title })).toHaveAttribute(
        'href',
        `/questions/${original.id}/${original.slug}`
    );
    await expect(mod.getByRole('link', { name: 'Reply' })).toHaveCount(0);

    // The asker sees the banner, cannot reply, and has no Close control.
    await user.goto(`/questions/${dup.id}/${dup.slug}`);
    await expect(user.locator('#closedNotice')).toBeVisible({ timeout: 15000 });
    await expect(user.getByRole('link', { name: 'Reply' })).toHaveCount(0);
    await expect(user.getByRole('link', { name: /^(Close|Reopen)$/ })).toHaveCount(0);
    expect((await postAnswer(user, dup.id, 'An answer to a closed question, long enough.')).status()).toBe(409);

    // Reopen, then close again with nothing entered: closed, no duplicate.
    mod.once('dialog', (d) => d.accept());
    await mod.getByRole('link', { name: 'Reopen' }).click();
    await expect(notice).toHaveCount(0);
    await expect(mod.getByRole('link', { name: 'Reply' }).first()).toBeVisible();
    mod.once('dialog', (d) => d.accept(''));
    await mod.getByRole('link', { name: 'Close' }).click();
    await expect(notice).toContainText('closed and takes no new answers');

    // Something that is not a link or an id is refused, and nothing changes.
    mod.once('dialog', (d) => d.accept());
    await mod.getByRole('link', { name: 'Reopen' }).click();
    await expect(notice).toHaveCount(0);
    const dialogs: string[] = [];
    mod.on('dialog', (d) => {
        dialogs.push(d.message());
        d.accept(dialogs.length === 1 ? 'not a question' : undefined);
    });
    await mod.getByRole('link', { name: 'Close' }).click();
    await expect.poll(() => dialogs.length).toBe(2);
    expect(dialogs[1]).toContain('does not look like a question');
    await expect(notice).toHaveCount(0);
});

test('a moderator deletes an answered question; its author cannot', async ({ browser }) => {
    const user = await (await browser.newContext()).newPage();
    await signInViaApi(user, 'dlu');
    const q = await createQuestion(user, tag);
    const res = await postAnswer(user, q.id, 'An answer that will go with its question.');
    const answerId = (await res.json()).data as string;

    await user.goto(`/questions/${q.id}/${q.slug}`);
    await expect(user.getByText('An answer that will go with its question.')).toBeVisible({ timeout: 15000 });
    await expect(user.locator(`a[href="/delete/${q.id}"]`)).toHaveCount(0);

    const mod = await (await browser.newContext()).newPage();
    await signInAsModerator(mod, 'dlm');
    await mod.goto(`/questions/${q.id}/${q.slug}`);
    // A moderator is offered Delete on someone else's answer and question.
    await expect(mod.locator(`a[href="/delete/${answerId}"]`)).toBeVisible({ timeout: 15000 });
    mod.once('dialog', (d) => {
        expect(d.message()).toContain('1 answer');
        d.accept();
    });
    await mod.locator(`a[href="/delete/${q.id}"]`).click();
    await expect(mod).toHaveURL(/\/questions$/, { timeout: 15000 });
    expect((await mod.request.get(`${BACKEND}/questions/${q.id}/${q.slug}`)).status()).not.toBe(200);
});

test('a moderator suspends and reinstates a user from their profile', async ({ browser }) => {
    const user = await (await browser.newContext()).newPage();
    const u = await signInViaApi(user, 'spu');
    const q = await createQuestion(user, tag);

    const mod = await (await browser.newContext()).newPage();
    const m = await signInAsModerator(mod, 'spm');

    // Ordinary users get no such button, on anyone's profile.
    await user.goto(`/users/${m.id}/${m.username}`);
    await expect(user.getByRole('heading', { name: m.username })).toBeVisible({ timeout: 15000 });
    await expect(user.getByRole('button', { name: /Suspend user|Reinstate user/ })).toHaveCount(0);

    await mod.goto(`/users/${u.id}/${u.username}`);
    mod.once('dialog', (d) => d.accept());
    await mod.getByRole('button', { name: 'Suspend user' }).click();
    await expect(mod.locator('#suspendedBadge')).toBeVisible();

    // The suspended user's session stops working, and they cannot sign in again.
    expect((await postAnswer(user, q.id, 'An answer from a suspended user, long enough.')).status()).toBe(403);
    const code = await user.request.post(`${BACKEND}/submit-email`, { data: { email: u.email } });
    expect(code.status()).toBe(403);

    // The state survives a reload; reinstating restores access.
    await mod.reload();
    await expect(mod.locator('#suspendedBadge')).toBeVisible({ timeout: 15000 });
    mod.once('dialog', (d) => d.accept());
    await mod.getByRole('button', { name: 'Reinstate user' }).click();
    await expect(mod.locator('#suspendedBadge')).toHaveCount(0);
    expect((await postAnswer(user, q.id, 'An answer after being reinstated, long enough.')).status()).toBe(200);

    // A moderator has no button on their own profile.
    await mod.goto(`/users/${m.id}/${m.username}`);
    await expect(mod.getByRole('heading', { name: m.username })).toBeVisible({ timeout: 15000 });
    await expect(mod.getByRole('button', { name: /Suspend user|Reinstate user/ })).toHaveCount(0);
});
