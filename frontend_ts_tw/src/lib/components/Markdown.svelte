<script lang="ts">
    import { highlightWithin, typesetMath } from '#lib/markdown.ts';

    // `html` must already be sanitized (see parseMarkdown).
    let { html, math = false, id }: { html: string; math?: boolean; id?: string } = $props();

    let root: HTMLDivElement;

    $effect(() => {
        void html;
        if (!root) return;
        highlightWithin(root);
        if (math) typesetMath(root);
    });
</script>

<div bind:this={root} {id} class="markdown prose prose-sm max-w-none prose-pre:bg-gray-50 prose-pre:text-gray-800 prose-code:before:content-none prose-code:after:content-none">
    {@html html}
</div>
