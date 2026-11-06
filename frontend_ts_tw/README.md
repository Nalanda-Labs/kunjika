# Kunjika frontend (SvelteKit + TypeScript + Tailwind)

A port of `../frontend` (SvelteKit 2, JavaScript, Bootstrap) to SvelteKit 3,
Svelte 5 runes, TypeScript and Tailwind CSS 4. It talks to the same backend
API and has the same pages, routes and features.

## Running

```bash
pnpm install
pnpm dev                 # http://localhost:5173
pnpm build && node build # production server (adapter-node), port 3000
pnpm check               # svelte-check / TypeScript
```

The backend URL is set in `src/lib/config.ts`. The backend's CORS list allows
`http://localhost:5173` and `http://localhost:3000`.

## Tests

The Playwright suite from `../frontend` runs unchanged against this app:

```bash
npx playwright install chromium
pnpm test:e2e   # needs the backend on :8000 (template.test.json) and Mailpit on :8026
```

`playwright.config.ts` builds the app and serves it on port 5173.

## Layout

| Path | Contents |
|---|---|
| `src/routes/` | Same routes as the original frontend |
| `src/lib/api.ts` | Typed fetch wrapper; sends the XSRF header automatically |
| `src/lib/types.ts` | Types for the API's responses |
| `src/lib/markdown.ts`, `editor.ts` | Markdown rendering (marked + DOMPurify + highlight.js + MathJax) and editor commands |
| `src/lib/components/` | Shared components: navigation, editor, tag input, dialogs, paged question list, profile tabs |
| `src/lib/server/cookies.ts` | Relays the backend's session cookies in the login, logout and delete-account actions |
| `src/app.css` | Tailwind setup, theme colours, and styles for rendered posts |

## SvelteKit 3 notes

- Kit configuration lives in `vite.config.ts` (`sveltekit({...})`); there is no `svelte.config.js`.
- `$lib` imports are `#lib/...` (Node subpath imports, declared in `package.json`).
- `$app/environment` is `$app/env`; page state comes from `$app/state`.
- `src/env.js` is reserved by Kit, so the API URL lives in `src/lib/config.ts`.

## Differences from the original

Fixes and replacements made while porting:

- Editor toolbar insertions update the bound value, so they're no longer lost when the form is
  submitted. Toolbar commands wrap the selected text.
- Image upload from the editor works; the old helper was missing its imports.
- "Share" copies a link to the answer instead of linking to a `/share/{id}` page that never
  existed. Report-queue links go to the real question URL.
- The confirm-email page offers "Resend" for expired links, as the backend's message says.
- Question pages load answers 50 at a time with a "Load more answers" button; the old page
  showed only the first 50.
- Icons are inline SVG instead of Font Awesome and Material Icons from CDNs; `svelte-tags-input`
  and `markdown-to-txt` are replaced by small local code.
- Form actions redirect with 303 after a POST.
