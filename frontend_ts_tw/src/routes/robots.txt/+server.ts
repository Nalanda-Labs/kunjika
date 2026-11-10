import { api_base } from '#lib/config.ts';
import type { RequestHandler } from './$types';

// Relayed from the backend, which knows the site's public host and so can
// write the absolute Sitemap URL. If it is unreachable, fall back to the
// permissive default without a Sitemap line.
export const GET: RequestHandler = async ({ fetch }) => {
    let body = '# https://www.robotstxt.org/robotstxt.html\nUser-agent: *\nDisallow:\n';
    try {
        const res = await fetch(`${api_base}/robots.txt`);
        if (res.ok) body = await res.text();
    } catch {
        // keep the default
    }
    return new Response(body, { headers: { 'Content-Type': 'text/plain; charset=utf-8' } });
};
