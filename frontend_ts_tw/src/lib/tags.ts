/**
 * The first tag that repeats in `tags`, compared case-insensitively and
 * ignoring surrounding whitespace, or `null`.
 *
 * `tags.name` is `citext` in the database, so "Rust" and "rust" are the same
 * tag; the API rejects such a list, and this lets the form say so first.
 */
export function firstDuplicateTag(tags: string[]): string | null {
    const seen = new Set<string>();
    for (const tag of tags) {
        const key = tag.trim().toLowerCase();
        if (seen.has(key)) return tag;
        seen.add(key);
    }
    return null;
}
