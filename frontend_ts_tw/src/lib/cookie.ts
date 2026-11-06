/** Read a cookie visible to page scripts (only non-httpOnly ones, e.g. `xsrf_token`). */
export default function getCookie(name: string): string {
    if (typeof document === 'undefined') return '';
    const prefix = `${name}=`;
    for (const part of decodeURIComponent(document.cookie).split(';')) {
        const c = part.trimStart();
        if (c.startsWith(prefix)) return c.slice(prefix.length);
    }
    return '';
}
