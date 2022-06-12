import * as cookie from 'cookie';
import { parseJwt } from '$lib/utils';

export async function handle({ event, resolve }) {
	const cookies = cookie.parse(event.request.headers.get('cookie') || '');
	jwt_decoded = parseJwt(cookies.jwt);
	if (jwt_decoded) {
		event.locals.user = jwt_decoded.user;
	} else {
		event.locals.user = null;
	}
	return await resolve(event);
}

export function getSession({ locals }) {
	return {
		user: locals.user && {
			username: locals.user.username,
			email: locals.user.email,
			id: locals.user.id,
			xsrf_token: locals.user.xsrf_token
		}
	};
}
