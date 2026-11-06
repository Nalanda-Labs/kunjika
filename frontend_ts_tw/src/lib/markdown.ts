import DOMPurify from 'dompurify';
import hljs from 'highlight.js';
import { marked } from 'marked';

marked.setOptions({ gfm: true, breaks: false });

/** Render markdown to sanitized HTML. Browser only (DOMPurify needs a DOM). */
export function parseMarkdown(text: string): string {
    if (typeof window === 'undefined') return '';
    return DOMPurify.sanitize(marked.parse(text ?? '', { async: false }));
}

/** Highlight code blocks inside `root` and label each with its language. */
export function highlightWithin(root: HTMLElement): void {
    for (const block of root.querySelectorAll<HTMLElement>('pre code')) {
        if (block.dataset.highlighted === 'yes') continue;
        hljs.highlightElement(block);
        const lang = [...block.classList]
            .find((c) => c.startsWith('language-'))
            ?.slice('language-'.length);
        if (lang && !block.querySelector('small.language')) {
            const small = document.createElement('small');
            small.className = `language ${lang}`;
            small.innerText = lang;
            block.appendChild(small);
        }
    }
}

// MathJax is loaded from its CDN on pages that render posts.
declare global {
    interface Window {
        MathJax?: {
            startup?: { promise?: Promise<unknown>; defaultReady?: () => void };
            typesetPromise?: (els?: HTMLElement[]) => Promise<unknown>;
            [k: string]: unknown;
        };
    }
}

/** Typeset TeX in `root` once MathJax has loaded (no-op if it never does). */
export function typesetMath(root?: HTMLElement): void {
    const mj = window.MathJax;
    mj?.startup?.promise
        ?.then(() => mj.typesetPromise?.(root ? [root] : undefined))
        .catch(() => {});
}

/** Inject the MathJax v4 loader once, with `$…$` inline math enabled. */
export function loadMathJax(): void {
    if (typeof window === 'undefined' || document.getElementById('mathjax-script')) return;
    window.MathJax = {
        tex: {
            inlineMath: [
                ['$', '$'],
                ['\\(', '\\)']
            ],
            displayMath: [
                ['$$', '$$'],
                ['\\[', '\\]']
            ]
        },
        startup: {
            ready: () => {
                const mj = window.MathJax!;
                mj.startup!.defaultReady!();
                mj.startup!.promise!.then(() => mj.typesetPromise?.());
            }
        }
    } as Window['MathJax'];
    const s = document.createElement('script');
    s.id = 'mathjax-script';
    s.defer = true;
    s.src = 'https://cdn.jsdelivr.net/npm/mathjax@4/tex-chtml.js';
    document.head.appendChild(s);
}

/** Plain text of a markdown document (browser only), e.g. for previews. */
export function markdownToText(text: string): string {
    if (typeof window === 'undefined') return '';
    const div = document.createElement('div');
    div.innerHTML = parseMarkdown(text);
    return (div.textContent ?? '').replace(/\s+/g, ' ').trim();
}
