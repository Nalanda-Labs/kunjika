import { api_base } from '#lib/config.ts';
import type { RequestHandler } from './$types';

// The sitemap has to live on the site's own origin for search engines to
// accept it, so this relays the one the backend generates.
export const GET: RequestHandler = async ({ fetch }) => {
    const res = await fetch(`${api_base}/sitemap.xml`);
    if (!res.ok) return new Response('Sitemap unavailable', { status: 502 });
    return new Response(await res.text(), {
        headers: {
            'Content-Type': 'application/xml; charset=utf-8',
            'Cache-Control': 'public, max-age=3600'
        }
    });
};
