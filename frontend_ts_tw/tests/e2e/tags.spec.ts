import { expect, test } from '@playwright/test';
import { BACKEND, anyTag, signInViaApi, uniqueUsername, xsrfHeaders } from './helpers';

// Tag pages and the moderator-only tag-info editing rule.

let tag: string;

test.beforeAll(async ({ request }) => {
    const t = await anyTag(request);
    test.skip(!t, 'no tags in the database');
    tag = t!;
});

test('tag info page renders for anonymous visitors without an edit button', async ({ page }) => {
    await page.goto(`/tags/info/${encodeURIComponent(tag)}`);
    await expect(page.getByRole('heading', { name: new RegExp(`About ${tag}`, 'i') })).toBeVisible();
    await expect(page.getByRole('button', { name: /Update Info/i })).toHaveCount(0);
});

test('regular users see no edit button and are sent away from the edit page', async ({ page }) => {
    await signInViaApi(page, 'tag');

    await page.goto(`/tags/info/${encodeURIComponent(tag)}`);
    await expect(page.getByRole('heading', { name: new RegExp(`About ${tag}`, 'i') })).toBeVisible();
    await expect(page.getByRole('button', { name: /Update Info/i })).toHaveCount(0);

    await page.goto(`/tags/edit/${encodeURIComponent(tag)}`);
    await expect(page).toHaveURL(/\/tags$/);
});

test('the API refuses tag-info edits from regular users', async ({ page, request }) => {
    await signInViaApi(page, 'tag');
    const before = await (
        await request.get(`${BACKEND}/get-tag-info/${encodeURIComponent(tag)}`)
    ).json();

    const res = await page.request.post(`${BACKEND}/tags/edit/${encodeURIComponent(tag)}`, {
        data: { tag_info: 'Vandalised by a regular user. '.repeat(10) },
        headers: await xsrfHeaders(page)
    });
    expect(res.status()).toBe(403);

    const after = await (
        await request.get(`${BACKEND}/get-tag-info/${encodeURIComponent(tag)}`)
    ).json();
    expect(after.info, 'tag info unchanged').toBe(before.info);
});

test('anonymous tag-info edits are rejected', async ({ request }) => {
    const res = await request.post(`${BACKEND}/tags/edit/${encodeURIComponent(tag)}`, {
        data: { tag_info: 'x'.repeat(300) }
    });
    expect(res.status()).toBeGreaterThanOrEqual(400);
    expect(res.status()).toBeLessThan(500);
});

test('an unknown tag returns 404', async ({ request }) => {
    const res = await request.get(`${BACKEND}/get-tag-info/${uniqueUsername('nosuch')}`);
    expect(res.status()).toBe(404);
});

test('questions tagged page loads from the live backend', async ({ page }) => {
    const statuses: number[] = [];
    page.on('response', (r) => {
        if (r.url().includes(':8000/api/v1/questions/tagged/')) statuses.push(r.status());
    });
    await page.goto(`/questions/tagged/${encodeURIComponent(tag)}`);
    await expect.poll(() => statuses.length).toBeGreaterThan(0);
    expect(statuses.every((s) => s < 500)).toBe(true);
});

test('tag search finds an existing tag', async ({ request }) => {
    const res = await request.post(`${BACKEND}/search-tags`, {
        data: { tag: tag.slice(0, 3), tags_per_page: 20 }
    });
    expect(res.status()).toBe(200);
    const names = ((await res.json()).data as Array<{ name: string }>).map((t) => t.name);
    expect(names).toContain(tag);
});
