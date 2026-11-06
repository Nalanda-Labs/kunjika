import { api_base } from './config.ts';
import getCookie from './cookie.ts';

type Method = 'GET' | 'POST';

export interface RequestOptions {
    /** XSRF token; in the browser it defaults to the `xsrf_token` cookie. */
    xsrf?: string;
    /** Incoming request headers whose `cookie` should be forwarded (server-side calls). */
    headers?: Headers | null;
}

async function send(
    method: Method,
    path: string,
    body: BodyInit | undefined,
    contentType: string | undefined,
    { xsrf, headers }: RequestOptions = {}
): Promise<Response> {
    const h: Record<string, string> = {};
    if (contentType) h['Content-Type'] = contentType;

    const token = xsrf ?? getCookie('xsrf_token');
    if (token) h['X-XSRF-TOKEN'] = token;

    if (path === 'auth/refresh') {
        h['Cache-Control'] = 'no-cache, no-store, must-revalidate';
        h['Pragma'] = 'no-cache';
        h['Expires'] = '0';
    }

    // Server-side calls have no cookie jar: forward the browser's cookies.
    const cookie = headers?.get('cookie');
    if (cookie) h['cookie'] = cookie;

    return fetch(`${api_base}/${path}`, { method, headers: h, body, credentials: 'include' });
}

export function get(path: string, opts?: RequestOptions): Promise<Response> {
    return send('GET', path, undefined, undefined, opts);
}

export function post(path: string, data?: unknown, opts?: RequestOptions): Promise<Response> {
    return send(
        'POST',
        path,
        data === undefined ? undefined : JSON.stringify(data),
        data === undefined ? undefined : 'application/json',
        opts
    );
}

/** Multipart upload. The browser sets the multipart boundary itself. */
export function upload(path: string, form: FormData, opts?: RequestOptions): Promise<Response> {
    return send('POST', path, form, undefined, opts);
}

/** Parse a response body as JSON; an empty body becomes `{}`. */
export async function json<T = Record<string, any>>(res: Response): Promise<T> {
    const text = await res.text();
    return (text ? JSON.parse(text) : {}) as T;
}

/** The fixed message the backend sends when a list query matches nothing. */
export const NO_ROWS = 'no rows returned by a query that expected to return at least one row';
