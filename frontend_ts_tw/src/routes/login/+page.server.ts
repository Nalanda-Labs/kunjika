import { fail, redirect } from '@sveltejs/kit';
import * as api from '#lib/api.ts';
import { relaySetCookies } from '#lib/server/cookies.ts';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = ({ locals }) => {
    if (locals.user) redirect(307, '/questions');
};

export const actions: Actions = {
    login: async ({ cookies, request }) => {
        const data = await request.formData();
        const email = String(data.get('email') ?? '');
        const res = await api.post('auth/login', { email, otp: String(data.get('sign_in_code') ?? '') });

        if (res.status !== 200) {
            return fail(res.status === 401 ? 401 : 400, {
                errors: 'Either the email or the sign-in code is wrong, or the code has expired.',
                email
            });
        }

        relaySetCookies(res, cookies);
        redirect(303, '/questions');
    }
};
