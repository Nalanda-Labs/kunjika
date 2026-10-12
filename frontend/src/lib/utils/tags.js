/**
 * Return the first tag that repeats in `tags`, or `null`.
 *
 * Comparison is case-insensitive and whitespace-trimmed because `tags.name`
 * is `citext` in the schema: "Rust" and "rust" name the SAME tag. The
 * `svelte-tags-input` `onlyUnique` prop is case-SENSITIVE, so it lets
 * ["rust", "Rust"] through and the API rejects it - this mirrors the server
 * rule so the user finds out before the round-trip.
 */
export function firstDuplicateTag(tags) {
    const seen = new Set();
    for (const tag of tags) {
        const key = tag.trim().toLowerCase();
        if (seen.has(key)) return tag;
        seen.add(key);
    }
    return null;
}
