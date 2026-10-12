import { defineConfig, devices } from '@playwright/test';

/**
 * ADDITIVE-ONLY Playwright config for the e2e suite.
 *
 * The repository already ships `playwright.config.ts`; it is left untouched.
 * That config has no `baseURL` (so `page.goto('/')` cannot resolve) and starts
 * the app on port 4173, whose origin the backend's CORS allow-list rejects
 * (`Cors::allowed_origin("http://localhost:5173")` in backend/src/main.rs).
 *
 * Run with:
 *   npx playwright test -c playwright.e2e.config.ts
 *
 * Requires the backend to be listening on 127.0.0.1:8000 - see TESTING.md.
 */
const PORT = 5173;
const BASE_URL = `http://localhost:${PORT}`;

export default defineConfig({
    testDir: 'tests/e2e',
    // Keep run artifacts inside tests/e2e/ so the repo-root .gitignore (which we
    // must not modify) does not need a new entry.
    outputDir: 'tests/e2e/.artifacts',
    testMatch: /(.+\.)?(test|spec)\.[jt]s/,
    fullyParallel: false,
    forbidOnly: !!process.env.CI,
    retries: process.env.CI ? 1 : 0,
    reporter: [['list']],
    timeout: 60_000,
    expect: { timeout: 15_000 },
    use: {
        baseURL: BASE_URL,
        trace: 'retain-on-failure',
        screenshot: 'only-on-failure'
    },
    projects: [{ name: 'chromium', use: { ...devices['Desktop Chrome'] } }],
    webServer: {
        command: `npm run build && npm run preview -- --port ${PORT} --strictPort`,
        url: BASE_URL,
        reuseExistingServer: !process.env.CI,
        timeout: 240_000
    }
});
