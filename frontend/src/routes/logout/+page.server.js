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
				const httpOnly = true;
				const secure = true;
				let path = '';
				let domain = '';
				let maxAge = 0;
				for (let cookie of split_cookie) {
					let i = cookie.indexOf('=');
					const cookie_name = cookie.slice(0, i).trim();
					const cookie_value = cookie.slice(i + 1).trim();

					if (cookie_name !== 'xsrf_token') {
						cookies.set(cookie_name, cookie_value, {
							httpOnly: httpOnly,
							domain: request.hostname,
							maxAge: 0,
							path: '/',
							secure: secure,
							sameSite: 'None'
						});
					} else {
						cookies.set(cookie_name, cookie_value, {
							httpOnly: false,
							domain: request.hostname,
							maxAge: 0,
							path: '/',
							secure: secure,
							sameSite: 'None'
						});
					}
				}

			}
		}

		locals.user = null;

		throw redirect(307, '/questions');
	}
}