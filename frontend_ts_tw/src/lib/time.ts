/**
 * Human-readable age of an epoch-seconds timestamp, e.g. "asked 5m ago",
 * "modified yesterday", "asked on 6/11/2026". `verb` may be empty.
 */
export function timeAgo(epochSecs: number, verb = '', now: number = Date.now()): string {
    const prefix = verb ? `${verb} ` : '';
    const when = new Date(epochSecs * 1000);
    const secs = Math.max(0, Math.floor((now - when.getTime()) / 1000));

    if (secs >= 3 * 86400) {
        const on = verb ? `${verb} on ` : '';
        return `${on}${when.getDate()}/${when.getMonth() + 1}/${when.getFullYear()}`;
    }
    if (secs >= 2 * 86400) return `${prefix}2 days ago`;
    if (secs >= 86400) return `${prefix}yesterday`;
    if (secs >= 3600) return `${prefix}${Math.floor(secs / 3600)}h ago`;
    if (secs >= 60) return `${prefix}${Math.floor(secs / 60)}m ago`;
    return `${prefix}${secs}s ago`;
}

/** "asked …" when a question was never edited, otherwise "modified …". */
export function listAge(cat: number, uat: number): string {
    return cat === uat ? timeAgo(cat, 'asked') : timeAgo(uat, 'modified');
}

/**
 * The RFC 3339 pagination cursor for an epoch-seconds timestamp. Going
 * backwards, one second is added: the database keeps sub-second precision
 * that epoch seconds drop, so without it the boundary row would repeat.
 */
export function cursor(epochSecs: number, back = false): string {
    return new Date((epochSecs + (back ? 1 : 0)) * 1000).toISOString();
}
