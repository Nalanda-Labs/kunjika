<script lang="ts">
    import Icon from './Icon.svelte';

    let {
        page,
        pages,
        onfirst,
        onprev,
        onnext
    }: {
        page: number;
        pages: number;
        onfirst: () => void;
        onprev: () => void;
        onnext: () => void;
    } = $props();

    const btn =
        'inline-flex h-8 w-8 items-center justify-center rounded hover:bg-gray-100 disabled:cursor-default disabled:text-gray-300 disabled:hover:bg-transparent';
</script>

<nav class="pagination mt-5 flex items-center justify-end gap-1 text-sm" aria-label="Pagination">
    <span class="mr-2 text-gray-600">Page {page} of {Math.max(pages, 1)}</span>
    <button type="button" class={btn} disabled={page <= 1} onclick={onfirst} aria-label="First page"
        ><Icon name="chevrons-left" /></button
    >
    <button
        type="button"
        class={btn}
        disabled={page <= 1}
        onclick={onprev}
        aria-label="Previous page"><Icon name="chevron-left" /></button
    >
    <button
        type="button"
        class={btn}
        disabled={page >= pages}
        onclick={onnext}
        aria-label="Next page"><Icon name="chevron-right" /></button
    >
</nav>
