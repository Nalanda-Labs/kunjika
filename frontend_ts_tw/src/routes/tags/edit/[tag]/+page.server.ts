import { redirect } from '@sveltejs/kit';
import { isReviewer } from '#lib/types.ts';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = ({ locals }) => {
    if (!locals.user) redirect(307, '/questions');
    // Only moderators can edit tag info; the backend enforces this too.
    if (!isReviewer(locals.user)) redirect(307, '/tags');
};
