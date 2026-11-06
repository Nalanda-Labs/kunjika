<script lang="ts">
    import { page } from '$app/state';
    import { onMount } from 'svelte';
    import * as api from '#lib/api.ts';
    import Markdown from '#lib/components/Markdown.svelte';
    import { parseMarkdown } from '#lib/markdown.ts';
    import { isReviewer } from '#lib/types.ts';

    let tag = $derived(page.params.tag ?? '');
    let html = $state('');
    let error = $state('');

    onMount(async () => {
        const res = await api.get(`get-tag-info/${encodeURIComponent(tag)}`);
        const j = await api.json<{ info?: string; message?: string }>(res);
        if (res.status === 200) html = parseMarkdown(j.info ?? '');
        else error = j.message || 'Could not load the tag info.';
    });
</script>

<svelte:head>
    <title>Tag Info ❤ Kunjika</title>
</svelte:head>

<div class="mx-auto mt-5 max-w-3xl">
    <h4 class="text-xl font-bold">About {tag}</h4>
    <h5 class="mt-1 text-gray-600">Tag Info</h5>
    <hr class="my-3 border-gray-200" />
    {#if error}<div class="alert-danger">{error}</div>{/if}
    <Markdown {html} />
    {#if isReviewer(page.data.user)}
        <a class="btn-primary mt-4" href="/tags/edit/{encodeURIComponent(tag)}">Update Info</a>
    {/if}
</div>
