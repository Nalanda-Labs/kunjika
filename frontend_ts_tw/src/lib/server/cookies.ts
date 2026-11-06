import type { Cookies } from '@sveltejs/kit';

/**
 * Copy the backend's `Set-Cookie` headers onto the frontend's domain.
 *
 * The backend sets its session cookies for `config.host`; when the frontend
 * is served from a different port they must be re-issued here. The cookie
 * `Domain` must not contain a port, so it is stripped. Every cookie except
 * `xsrf_token` (which page scripts read) is httpOnly.
 */
export function relaySetCookies(res: Response, cookies: Cookies): void {
    for (const header of res.headers.getSetCookie()) {
        const [pair, ...attrs] = header.split(';');
        const eq = pair.indexOf('=');
        if (eq < 0) continue;
        const name = pair.slice(0, eq).trim();
        const value = pair.slice(eq + 1);

        let path = '/';
        let domain: string | undefined;
        let maxAge: number | undefined;
        for (const attr of attrs) {
            const i = attr.indexOf('=');
            if (i < 0) continue;
            const key = attr.slice(0, i).trim().toLowerCase();
            const v = attr.slice(i + 1).trim();
            if (key === 'path') path = v || '/';
            else if (key === 'domain') domain = v.split(':')[0] || undefined;
            else if (key === 'max-age') maxAge = parseInt(v, 10);
        }

        cookies.set(name, value, {
            path,
            domain,
            maxAge,
            httpOnly: name !== 'xsrf_token',
            secure: true,
            sameSite: 'lax'
        });
    }
}

/** Remove every cookie the browser sent, on both the bare and the explicit domain. */
export function clearAllCookies(request: Request, cookies: Cookies, url: URL): void {
    for (const part of (request.headers.get('cookie') ?? '').split(';')) {
        const name = part.split('=')[0]?.trim();
        if (!name) continue;
        cookies.delete(name, { path: '/' });
        cookies.delete(name, { path: '/', domain: url.hostname });
    }
}
