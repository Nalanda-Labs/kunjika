import { fail, redirect } from '@sveltejs/kit';
import * as api from '#lib/api.ts';
import { clearAllCookies } from '#lib/server/cookies.ts';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = () => {
    redirect(302, '/');
};

export const actions: Actions = {
    default: async ({ cookies, locals, request, url }) => {
        const res = await api.post('auth/logout', {}, {
            xsrf: cookies.get('xsrf_token') ?? '',
            headers: request.headers
        });
        if (res.status !== 200) {
            return fail(res.status, await api.json(res));
        }
        clearAllCookies(request, cookies, url);
        locals.user = null;
        redirect(303, '/questions');
    }
};
