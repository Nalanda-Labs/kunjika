import { expect, test } from '@playwright/test';

// ADDITIVE-ONLY e2e: new files under frontend/tests/e2e/.
// Existing frontend/tests/test.ts, playwright.config.ts and src/* untouched.
// Backend must be reachable at http://localhost:8000/api/v1 (see TESTING.md).

test('home page renders hero', async ({ page }) => {
	await page.goto('/');
	await expect(page.getByRole('heading', { name: /Ask anything/i })).toBeVisible();
});

test('questions page loads from the live backend', async ({ page }) => {
	const responses: number[] = [];
	
	page.on('response', (r) => {
		if (r.url().includes(':8000/api/v1')) responses.push(r.status());
	});
	
	await page.goto('/questions');
	await expect(page.getByRole('heading', { name: /questions/i }).first()).toBeVisible();
	// The list is fetched client-side, so the backend must actually answer.
	expect(responses.length).toBeGreaterThan(0);
	expect(responses.every((s) => s < 500)).toBe(true);
});

test('tags page loads', async ({ page }) => {
	const responses: number[] = [];
	
	page.on('response', (r) => {
		if (r.url().includes(':8000/api/v1')) responses.push(r.status());
	});
	
	await page.goto('/tags');
	await expect(page.getByRole('heading', { name: /tags/i }).first()).toBeVisible();
	expect(responses.length).toBeGreaterThan(0);
	expect(responses.every((s) => s < 500)).toBe(true);
});

test('users page loads', async ({ page }) => {
	const responses: number[] = [];
	
	page.on('response', (r) => {
		if (r.url().includes(':8000/api/v1')) responses.push(r.status());
	});
	
	await page.goto('/users');
	await expect(page.getByRole('heading', { name: /users/i }).first()).toBeVisible();
	
	expect(responses.length).toBeGreaterThan(0);
	expect(responses.every((s) => s < 500)).toBe(true);
});
