/** @type {import('./$types').LayoutServerLoad} */
export function load({ locals}) {
    return {
        user: locals.user && {
            username: locals.user.username,
            email: locals.user.email,
            firstName: locals.user.firstName,
            lastName: locals.user.lastName,
            isAdmin: locals.user.isAdmin
        },
        locals
    };
}
