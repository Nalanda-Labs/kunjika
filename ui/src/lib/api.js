import { apiUrl } from "./constants";
const base = 'http://localhost/api';

async function send({ method, path, data, xsrf_token }) {
	const opts = { method, headers: {} };

	if (data) {
		opts.headers['Content-Type'] = 'application/json';
		opts.body = JSON.stringify(data);
	}

	if(xsrf_token) {
		opts.headers['X-XSRF-Token'] = xsrf_token
	}
	opts.credentials = 'include';

	return fetch(`${base}/${path}`, opts)
		.then((r) => r.text())
		.then((json) => {
			try {
				return JSON.parse(json);
			} catch (err) {
				return json;
			}
		});
}

export function get(path, xsrf_token) {
	return send({ method: 'GET', path, xsrf_token });
}

export function del(path, xsrf_token) {
	return send({ method: 'DELETE', path, xsrf_token });
}

export function post(path, data, xsrf_token) {
	return send({ method: 'POST', path, data, xsrf_token });
}

export function put(path, data, xsrf_token) {
	return send({ method: 'PUT', path, data, xsrf_token });
}
