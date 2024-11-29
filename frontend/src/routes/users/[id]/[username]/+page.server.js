import { fail, redirect } from '@sveltejs/kit';
import * as api from '../../../../lib/api.js';

/** @type {import('./$types.js').Actions} */
export const actions = {
    default: async ({ cookies, locals, request }) => {
        let xsrf_token = cookies.get('xsrf_token');

        let response = await api.post(`delete-profile/${locals.user.id}`, {}, xsrf_token, request.headers);

        console.log(response.status);
        if (response.status === 200) {
            const resp = await api.post('auth/logout', {}, xsrf_token, request.headers)

            let text = await resp.text();
            let j = text ? JSON.parse(text) : {};

            // in case of delete profile backend wont find the user in redis and 401 will
            // be retruned
            if (resp.status === 401) {
                if(j.message === "the user belonging to this token no logger exists") {
                    console.log("User deleted!");
                }
                else {
                    return fail(resp.status, j);
                }
            }
        } else {
            return fail(response.status);
        }

        for (const pair of request.headers.entries()) {
            if (pair[0] === 'cookie') {
                let split_cookie = pair[1].split(';');
                split_cookie.forEach((c) => {
                    var i = c.indexOf('=');
                    var name = c.slice(0, i).trim();
                    cookies.delete(name, { path: '/' });
                });

            }
        }

        locals.user = null;

        throw redirect(307, '/questions');
    }
}