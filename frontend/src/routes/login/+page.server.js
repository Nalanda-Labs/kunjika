import { redirect } from '@sveltejs/kit';
import * as api from '../../lib/api.js';
import { fail } from '@sveltejs/kit';

/** @type {import('./$types').PageServerLoad} */
export async function load({ locals }) {
	if (locals.user) throw redirect(307, '/questions');
}

/** @type {import('./$types').Actions} */
export const actions = {
	login: async ({ cookies, request }) => {
		const data = await request.formData();

		const resp = await api.post('auth/login', {
			email: data.get('email'),
			password: data.get('password')
		});

		let text = await resp.text();
		let j = text ? JSON.parse(text) : {};

		if (resp.status === 401) {
			return fail(401, {
				success: false,
				errors: 'Either email or password is wrong or you have not verified your email!',
				email: data.get('email')
			});
		}

		for (const pair of resp.headers.entries()) {
			// this is not general cookie parsing logic but specific to what we use
			// we do not even try to parse http only and secure attributes and
			// force them to be always true as a best practice.
			if (pair[0] === 'set-cookie') {
				let split_cookie = pair[1].split(';');
				const httpOnly = true;
				const secure = true;
				let path = '';
				let domain = '';
				let maxAge = 0;

				let i = split_cookie[0].indexOf('=');
				const cookie_name = split_cookie[0].slice(0, i);
				const cookie_value = split_cookie[0].slice(i + 1);

				split_cookie.forEach((e) => {
					var i = e.indexOf('=');
					if (i !== -1) {
						let name = e.slice(0, i).trim();
						let value = e.slice(i + 1);
						console.log(name, value);
						if (name === 'Path') {
							path = value;
						} else if (name === 'Domain') {
							domain = value;
						} else if (name === 'Max-Age') {
							maxAge = parseInt(value);
							console.log(maxAge);
						}
					}
				});

				if (cookie_name !== 'xsrf_token') {
					cookies.set(cookie_name, cookie_value, {
						httpOnly: httpOnly,
						domain: domain,
						maxAge: maxAge,
						path: path,
						secure: secure,
						sameSite: 'lax'
					});
				} else {
					cookies.set(cookie_name, cookie_value, {
						httpOnly: false,
						domain: domain,
						maxAge: maxAge,
						path: path,
						secure: secure,
						sameSite: 'lax'
					});
				}
			}
		}

		throw redirect(307, '/questions');
	}
};
