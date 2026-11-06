/**
 * Markdown editor commands. Each takes the current text and selection and
 * returns the new text plus the range to select, so the caller can update
 * its bound value (writing to `textarea.value` directly would bypass it).
 */
export interface Edit {
    value: string;
    start: number;
    end: number;
}

export type Command = (value: string, start: number, end: number) => Edit;

/**
 * Wrap the selection in `before`/`after`. With nothing selected, insert
 * `placeholder` wrapped the same way and select it, ready to type over.
 */
function wrap(before: string, after: string, placeholder: string): Command {
    return (value, start, end) => {
        const selected = value.slice(start, end) || placeholder;
        const next = value.slice(0, start) + before + selected + after + value.slice(end);
        const s = start + before.length;
        return { value: next, start: s, end: s + selected.length };
    };
}

/** Insert `block` at the caret and select `select` inside it (if found). */
function insertBlock(block: string, select: string): Command {
    return (value, start, end) => {
        const next = value.slice(0, start) + block + value.slice(end);
        const at = block.indexOf(select);
        const s = start + (at >= 0 ? at : block.length);
        return { value: next, start: s, end: at >= 0 ? s + select.length : s };
    };
}

function admonition(kind: 'note' | 'tip' | 'warning', label: string): Command {
    return insertBlock(
        `\n<div class="admonition ${kind}">\n<span class="title"><b>${label}:</b></span>\n<p>Your text</p>\n</div>\n`,
        'Your text'
    );
}

export const bold = wrap('**', '**', 'bold text');
export const italic = wrap('*', '*', 'emphasized text');
export const link: Command = (value, start, end) => {
    const text = value.slice(start, end) || 'text';
    const block = `[${text}](link)`;
    const next = value.slice(0, start) + block + value.slice(end);
    const s = start + text.length + 3;
    return { value: next, start: s, end: s + 'link'.length };
};
export const unorderedList = insertBlock('\n- First item\n- Second item\n', 'First item');
export const orderedList = insertBlock('\n1. First item\n2. Second item\n', 'First item');
export const codeBlock = insertBlock('\n```language\n<code here>\n```\n', 'language');
export const note = admonition('note', 'Note');
export const tip = admonition('tip', 'Tip');
export const warning = admonition('warning', 'Warning');

export function insertText(text: string): Command {
    return (value, start, end) => {
        const next = value.slice(0, start) + text + value.slice(end);
        const s = start + text.length;
        return { value: next, start: s, end: s };
    };
}

/** Keyboard shortcuts: Ctrl/Cmd + B, I, K. */
export function shortcutFor(e: KeyboardEvent): Command | null {
    if (!(e.ctrlKey || e.metaKey) || e.altKey) return null;
    switch (e.key.toLowerCase()) {
        case 'b':
            return bold;
        case 'i':
            return italic;
        case 'k':
            return link;
        default:
            return null;
    }
}
