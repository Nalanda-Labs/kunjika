/** @type {import('./$types').LayoutServerLoad} */
export function load({ locals}) {
    return {
        user: locals.user && {
            username: locals.user.username,
            id: locals.user.id,
            email: locals.user.email,
            firstName: locals.user.firstName,
            lastName: locals.user.lastName,
            isAdmin: locals.user.isAdmin,
            image_url: locals.user.image_url
        },
        locals
    };
}
