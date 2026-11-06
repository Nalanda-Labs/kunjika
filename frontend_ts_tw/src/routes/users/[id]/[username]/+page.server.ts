import { fail, redirect } from '@sveltejs/kit';
import * as api from '#lib/api.ts';
import { clearAllCookies } from '#lib/server/cookies.ts';
import type { Actions } from './$types';

export const actions: Actions = {
    // Delete your own account, then end the session.
    default: async ({ cookies, locals, request, url }) => {
        if (!locals.user) redirect(303, '/login');
        const opts = { xsrf: cookies.get('xsrf_token') ?? '', headers: request.headers };

        const res = await api.post(`delete-profile/${locals.user.id}`, {}, opts);
        if (res.status !== 200) return fail(res.status, await api.json(res));

        // The account is gone, so the backend rejects the session (401); the
        // logout call only clears what it can.
        await api.post('auth/logout', {}, opts).catch(() => {});
        clearAllCookies(request, cookies, url);
        locals.user = null;
        redirect(303, '/questions');
    }
};
