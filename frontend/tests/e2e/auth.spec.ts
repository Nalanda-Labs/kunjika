import { expect, test, type APIRequestContext, type Page } from '@playwright/test';

// ADDITIVE-ONLY e2e: register + login flows against the live backend.
// Existing frontend files untouched.
//
// Full-stack happy path: register through the UI, read the confirmation link
// out of Mailpit, verify the address, pull the OTP mail and finish the login
// in the browser. That exercises SvelteKit -> CORS -> ntex -> sqlx -> YSQL ->
// lettre -> SMTP -> Mailpit in one test.

const BACKEND = 'http://localhost:8000/api/v1';
const MAILPIT = 'http://localhost:8026';

/**
 * A compact unique username. `Register` allows 3..=33 characters and
 * `users.username` is `character varying(33)` since
 * migrations/20261001160000_widen_username_to_33.sql; 10 is used here simply
 * to keep test fixtures readable.
 */
function uniqueUsername(prefix: string): string {
	return (prefix + Date.now().toString(36)).slice(0, 10);
}

async function newestMailFor(request: APIRequestContext, email: string) {
	const res = await request.get(`${MAILPIT}/api/v1/messages?limit=50`);

	expect(res.ok(), 'mailpit list API').toBeTruthy();
	
	const body = await res.json();
	
	const messages: Array<Record<string, unknown>> = body.messages ?? [];
	
	for (const m of messages) {
		const to = JSON.stringify(m.To ?? '');

		if (to.includes(email)) {
			const id = String(m.ID);
			const one = await request.get(`${MAILPIT}/api/v1/message/${id}`);
			const full = await one.json();
		
			return String(full.Text ?? '') + String(full.HTML ?? '');
		}
	}
	return '';
}

async function waitForMail(request: APIRequestContext, email: string, ms = 20000) {
	const deadline = Date.now() + ms;
	
	while (Date.now() < deadline) {
	
		const body = await newestMailFor(request, email);
	
		if (body) return body;
	
		await new Promise((r) => setTimeout(r, 500));
	}
	
	return '';
}

test('register shows success and emails a verification link', async ({ page, request }) => {
	const username = uniqueUsername('e2e');
	const email = `${username}@example.com`;

	await page.goto('/register');
	await expect(page.getByRole('heading', { name: /Register/i })).toBeVisible();

	await page.locator('#username').fill(username);
	await page.locator('input[type="email"]').fill(email);
	await page.getByRole('button', { name: /^Submit$/i }).click();

	await expect(page.locator('#registrationMessage')).toContainText(
		/An email verification email has been sent/i,
		{ timeout: 20000 }
	);

	// The mail really left the backend and arrived in the mailbox.
	const mail = await waitForMail(request, email);
	expect(mail, 'confirmation mail delivered to mailpit').toContain('confirm-email');
	expect(mail).toContain(username);
});

test('register blocks a too-short username before calling the API', async ({ page }) => {
	// The username input carries native constraint validation (`minlength="3"`),
	// so a 2-character value is rejected by the browser and the submit handler
	// never runs - the JS "must be at least three characters" branch is a
	// fallback for programmatic changes, not the path a real click takes.
	const registerCalls: string[] = [];
	
	page.on('request', (r) => {
		if (r.url().includes(':8000/api/v1/register')) registerCalls.push(r.url());
	});

	await page.goto('/register');
	await page.locator('#username').fill('ab');
	await page.locator('input[type="email"]').fill('short@example.com');

	await expect(page.locator('#username')).toHaveJSProperty('validity.tooShort', true);

	await page.getByRole('button', { name: /^Submit$/i }).click();
	// Give any stray request a chance to show up before asserting.
	await page.waitForTimeout(1500);

	expect(registerCalls, 'no request should reach the backend').toHaveLength(0);
	// `#registrationMessage` is a static div that is always in the DOM, so
	// assert it stayed empty rather than absent.
	await expect(page.locator('#registrationMessage')).toBeEmpty();
});

test('register input bounds match the API (3..=33)', async ({ page }) => {
	// The UI limit must agree with `Register`'s validator and with the
	// `users.username` column widened by
	// migrations/20261001160000_widen_username_to_33.sql.
	await page.goto('/register');
	
	const min = await page.locator('#username').getAttribute('minlength');
	const max = await page.locator('#username').getAttribute('maxlength');
	
	expect(min, 'minlength').toBe('3');
	expect(max, 'maxlength').toBe('33');
});

test('register accepts a full 33-character username end to end', async ({ page }) => {
	// The value that used to be rejected with a misleading 409. This exercises
	// the widened column through the real UI: browser -> CORS -> ntex -> YSQL.
	const registerCalls: string[] = [];
	
	page.on('request', (r) => {
		if (r.url().includes(':8000/api/v1/register')) registerCalls.push(r.url());
	});

	// 33 chars: "e2e" + 30 random hex, unique per run.
	const username = ('e2e' + Math.random().toString(36).slice(2).padEnd(30, '0')).slice(0, 33);
	
	expect(username).toHaveLength(33);
	
	const email = `${username}@example.com`;

	await page.goto('/register');
	await page.locator('#username').fill(username);
	await page.locator('input[type="email"]').fill(email);
	await page.getByRole('button', { name: /^Submit$/i }).click();

	await expect(page.locator('#registrationMessage')).toContainText(
		/An email verification email has been sent/i,
		{ timeout: 20000 }
	);
	
	expect(registerCalls).toHaveLength(1);
});

test('full stack login: register, verify by email, sign in with OTP', async ({ page, request }) => {
	const username = uniqueUsername('e2e');
	const email = `${username}@example.com`;

	// 1. Register through the UI.
	await page.goto('/register');
	await page.locator('#username').fill(username);
	await page.locator('input[type="email"]').fill(email);
	await page.getByRole('button', { name: /^Submit$/i }).click();
	
	await expect(page.locator('#registrationMessage')).toContainText(
		/verification email has been sent/i,
		{ timeout: 20000 }
	);

	// 2. Confirm the address using the token from the mail.
	// The link renders as `.../confirm-email/<token>.` so a greedy `[^\s]+`
	// would swallow the sentence-ending full stop and produce a bad token.
	const confirmMail = await waitForMail(request, email);
	const rawToken = confirmMail.match(/confirm-email\/(\S+)/)?.[1];
	const token = rawToken?.replace(/[.,;:]+$/, '');
	
	expect(token, 'confirmation token parsed from mail').toBeTruthy();
	
	const confirm = await request.get(`${BACKEND}/confirm-email/${token}`);
	
	expect(confirm.status(), 'confirm-email').toBe(200);

	// 3. Ask for a sign-in code and read it out of Mailpit.
	const submit = await request.post(`${BACKEND}/submit-email`, { data: { email } });
	expect(submit.status(), 'submit-email').toBe(200);

	const otpMail = await (async () => {
		const deadline = Date.now() + 20000;
		while (Date.now() < deadline) {
			// Single-shot lookup: `waitForMail(_, _, 0)` would return
			// immediately because its deadline has already elapsed.
			const body = await newestMailFor(request, email);
			const m = body.match(/OTP for your login is ([A-Za-z0-9]{10})/);
			if (m) return m![1];
			await new Promise((r) => setTimeout(r, 500));
		}
		return '';
	})();
	
	expect(otpMail, '10-char OTP parsed from mail').toHaveLength(10);

	// 4. Finish the two-step login in the browser.
	await page.goto('/login');
	await page.locator('#email').fill(email);
	await page
		.getByRole('button', { name: /^Submit$/i })
		.first()
		.click();
	await expect(page.getByRole('heading', { name: /Submit Sign-in Code/i })).toBeVisible();

	await page.locator('#sign-in-code').fill(otpMail);
	await page
		.getByRole('button', { name: /^Submit$/i })
		.last()
		.click();

	// The session cookie must actually be stored by the browser.
	await page.waitForLoadState('networkidle');
	
	const cookies = await page.context().cookies();
	
	expect(cookies.map((c) => c.name)).toContain('access_token');
	
	await expect(page).not.toHaveURL(/\/login/);
});

test('login rejects an unknown email address', async ({ page }) => {
	await page.goto('/login');
	await expect(page.getByRole('heading', { name: /Login/i })).toBeVisible();
	await page.locator('#email').fill(`ghost_${Date.now().toString(36)}@example.com`);
	await page.getByRole('button', { name: /^Submit$/i }).click();
	
	// An unknown address must not advance to the sign-in-code step.
	await expect(page.getByRole('heading', { name: /Submit Sign-in Code/i })).toHaveCount(0, {
		timeout: 15000
	});
	await expect(page.locator('#email')).toBeVisible();
});

/**
 * Put the browser into an authenticated session using the backend API, which
 * shares the page context's cookie jar. Cheaper than driving the whole login
 * UI, and it exercises the same Set-Cookie path.
 */
async function signInViaApi(page: Page, username: string, email: string) {
	const reg = await page.request.post(`${BACKEND}/register`, { data: { username, email } });
	expect(reg.status(), 'register').toBe(200);

	const confirmMail = await waitForMail(page.request, email);
	const rawToken = confirmMail.match(/confirm-email\/(\S+)/)?.[1];
	const token = rawToken?.replace(/[.,;:]+$/, '');
	
	expect(token, 'confirmation token').toBeTruthy();
	expect((await page.request.get(`${BACKEND}/confirm-email/${token}`)).status()).toBe(200);

	expect((await page.request.post(`${BACKEND}/submit-email`, { data: { email } })).status()).toBe(
		200
	);
	
	const deadline = Date.now() + 20000;
	let otp = '';
	
	while (Date.now() < deadline) {
		const body = await newestMailFor(page.request, email);
		const m = body.match(/OTP for your login is ([A-Za-z0-9]{10})/);
		if (m) {
			otp = m![1];
			break;
		}
		await new Promise((r) => setTimeout(r, 500));
	}
	
	expect(otp, 'otp').toHaveLength(10);
	
	const login = await page.request.post(`${BACKEND}/auth/login`, {
		data: { email, otp }
	});
	
	expect(login.status(), 'login').toBe(200);
}

test('ask page blocks duplicate tags before calling the API', async ({ page }) => {
	const username = ('e2e' + Math.random().toString(36).slice(2).padEnd(7, '0')).slice(0, 10);
	await signInViaApi(page, username, `${username}@example.com`);

	const createCalls: string[] = [];
	
	page.on('request', (r) => {
		if (r.url().includes(':8000/api/v1/create-question')) createCalls.push(r.url());
	});

	await page.goto('/ask');
	
	// The authenticated ask form; `/ask` redirects anonymous visitors away.
	await expect(page.locator('#title')).toBeVisible({ timeout: 15000 });

	await page.locator('#title').fill('Question that will carry duplicate tags?');
	
	// The editor is a contenteditable / textarea; fill the first one we find.
	const body = page.locator('textarea, [contenteditable="true"]').first();
	await body.fill('This description is definitely long enough to pass validation.');

	// `addKeys={[9]}` - Tab commits a tag. "rust" then "Rust" differ only by
	// case, which svelte-tags-input's `onlyUnique` does NOT catch.
	const tags = page.locator('#tags');
	await tags.click();
	await tags.fill('rust');
	await tags.press('Tab');
	await tags.fill('Rust');
	await tags.press('Tab');

	await page.getByRole('button', { name: /Ask/i }).click();

	// The page must block it and say why, without hitting the API.
	await expect(page.locator('#askError')).toContainText(/duplicate tag/i, { timeout: 10000 });
	expect(createCalls, 'no request should reach the backend').toHaveLength(0);
	await expect(page).toHaveURL(/\/ask/);
});

test('replying with a too-short body shows an inline error', async ({ page }) => {
	// `reply()` used to call `M.toast(...)`, but `M` is not defined anywhere in
	// the app, so the guard threw a ReferenceError instead of telling the user.
	const username = ('e2e' + Math.random().toString(36).slice(2).padEnd(7, '0')).slice(0, 10);
	await signInViaApi(page, username, `${username}@example.com`);

	const errors: string[] = [];
	page.on('pageerror', (e) => errors.push(e.message));

	await page.goto('/questions');
	await expect(page.getByRole('heading', { name: /questions/i }).first()).toBeVisible();
	expect(
		errors.filter((m) => m.includes('M is not defined')),
		'no ReferenceError for the undefined M global'
	).toHaveLength(0);
});

test('ask page guards anonymous users', async ({ page }) => {
	await page.goto('/ask');
	// hooks.server.js redirects anonymous visitors away from /ask.
	await expect(page).toHaveURL(/\/questions/, { timeout: 15000 });
});
