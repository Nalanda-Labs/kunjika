/** @type {import('@sveltejs/kit').Handle} */
export function handle({ event, resolve }) {
    const jwt = event.cookies.get('logged_in');
    event.locals.user = jwt ? JSON.parse(jwt).user : null;

    return resolve(event);
}