import * as cookie from 'cookie';
import { parseJwt } from '$lib/utils';

export async function handle({ event, resolve }) {
	const cookies = cookie.parse(event.request.headers.get('cookie') || '');
	jwt_decoded = parseJwt(cookies.jwt);

	if (jwt_decoded !== '') {
		event.locals.email = jwt_decoded.email;
		event.locals.username = jwt_decoded.username;
		event.locals.id = jwt_decoded.id;
		event.locals.xsrf_token = jwt_decoded.xsrf_token;
	} else {
		event.locals.user = null;
	}
	return await resolve(event);
}

export function getSession({ locals }) {
	if (locals.username) {
		return {
			user: {
				username: locals.username,
				email: locals.email,
				xsrf_token: locals.xsrf_token,
				id: locals.id
			}
		}
	} else {
		return {user: null}
	}
}
