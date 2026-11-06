import type { Handle } from '@sveltejs/kit/hooks';

export const handle: Handle = ({ event, resolve }) => {
    // `logged_in` is a display-only snapshot of the user. It is not signed, so
    // it must never be used for authorization: every protected action is
    // checked by the backend against the signed access token. A malformed
    // value is treated as "logged out" instead of failing every page.
    const raw = event.cookies.get('logged_in');
    let user = null;
    if (raw) {
        try {
            user = JSON.parse(raw).user ?? null;
        } catch {
            user = null;
        }
    }
    event.locals.user = user;
    return resolve(event);
};
