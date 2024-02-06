import { fail, redirect } from '@sveltejs/kit';
import * as api from '../../lib/api.js';

export function load() {
	throw redirect(302, '/');
}

/** @type {import('./$types.js').Actions} */
export const actions = {
	default: async ({ cookies, locals, request }) => {
		let xsrf_token = cookies.get('xsrf_token');
		const resp = await api.post('auth/logout', {}, xsrf_token, request.headers)

		let text = await resp.text();
		let j = text ? JSON.parse(text) : {};

		if (j.errors || resp.status != 200) {
			return fail(resp.status, j);
		}

		for (const pair of request.headers.entries()) {
			if (pair[0] === 'cookie') {
				let split_cookie = pair[1].split(';');
				split_cookie.forEach((c) => {
					var i = c.indexOf('=');
					var name = c.slice(0, i);
					cookies.delete(name, {path: '/'});
				});

			}
		}

		locals.user = null;

		throw redirect(307, '/questions');
	}
}