import { redirect } from '@sveltejs/kit';

/** @type {import('./$types').PageServerLoad} */
export async function load({ locals }) {
    if (!locals.user) throw redirect(307, '/questions');
    // Only moderators can edit tag info; the backend enforces this too.
    if (!locals.user.is_moderator && !locals.user.is_superuser) throw redirect(307, '/tags');
}
