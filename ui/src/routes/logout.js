
export function post() {
    return {
        headers: {
            "set-cookie":
                'jwt=""; path=/; HttpOnly; expires=Thu, 01 Jan 1970 00:00:00 GMT',
        },
        body: {
            ok: true,
        },
    };
}