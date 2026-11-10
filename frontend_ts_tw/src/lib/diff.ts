/** One line of a diff: unchanged, added in the new text, or removed from the old. */
export interface DiffLine {
    kind: 'same' | 'add' | 'del';
    text: string;
}

/**
 * Line-by-line diff of two texts (longest common subsequence). Posts are at
 * most 100,000 characters, so the quadratic table stays small; texts with an
 * unusually large number of lines fall back to "everything changed" rather
 * than freezing the page.
 */
export function diffLines(before: string, after: string): DiffLine[] {
    const a = before.split('\n');
    const b = after.split('\n');
    if (a.length * b.length > 4_000_000) {
        return [
            ...a.map((text): DiffLine => ({ kind: 'del', text })),
            ...b.map((text): DiffLine => ({ kind: 'add', text }))
        ];
    }

    // lcs[i][j] = length of the longest common subsequence of a[i..] and b[j..].
    const lcs: Uint32Array[] = Array.from({ length: a.length + 1 }, () => new Uint32Array(b.length + 1));
    for (let i = a.length - 1; i >= 0; i--) {
        for (let j = b.length - 1; j >= 0; j--) {
            lcs[i][j] = a[i] === b[j] ? lcs[i + 1][j + 1] + 1 : Math.max(lcs[i + 1][j], lcs[i][j + 1]);
        }
    }

    const out: DiffLine[] = [];
    let i = 0;
    let j = 0;
    while (i < a.length && j < b.length) {
        if (a[i] === b[j]) {
            out.push({ kind: 'same', text: a[i] });
            i++;
            j++;
        } else if (lcs[i + 1][j] >= lcs[i][j + 1]) {
            out.push({ kind: 'del', text: a[i++] });
        } else {
            out.push({ kind: 'add', text: b[j++] });
        }
    }
    while (i < a.length) out.push({ kind: 'del', text: a[i++] });
    while (j < b.length) out.push({ kind: 'add', text: b[j++] });
    return out;
}
