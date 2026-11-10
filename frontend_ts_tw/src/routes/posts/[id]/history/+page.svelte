<script lang="ts">
    import { page } from '$app/state';
    import { onMount } from 'svelte';
    import * as api from '#lib/api.ts';
    import type { PostHistory } from '#lib/types.ts';

    // Edit history of one post: the current text, then every earlier version.
    let id = $derived(page.params.id ?? '');
    let history = $state<PostHistory | null>(null);
    let error = $state('');

    onMount(async () => {
        try {
            const res = await api.get(`posts/${id}/history`);
            const j = await api.json<{ data?: PostHistory; message?: string }>(res);
            if (res.status === 200 && j.data) history = j.data;
            else error = res.status === 404 ? 'This post does not exist.' : j.message || 'Could not load the history.';
        } catch {
            error = 'Could not load the history. Please try again.';
        }
    });

    const when = (secs: number) => new Date(secs * 1000).toLocaleString();
</script>

<svelte:head>
    <title>Edit history ❤ Kunjika</title>
</svelte:head>

<div class="mt-5">
    <h3 class="text-2xl font-normal">Edit history</h3>
    {#if error}<div class="alert-danger mt-3" role="alert">{error}</div>{/if}
    {#if history}
        <p class="mt-2 text-sm text-gray-500">
            {history.is_question ? 'Question' : 'Answer'} by
            <a class="link" href="/users/{history.posted_by_id}/{history.author}">{history.author}</a>
            ·
            <a
                class="link"
                href="/questions/{history.question_id}/{history.question_slug}{history.is_question ? '' : `#${id}`}"
                >back to the post</a
            >
        </p>

        <section class="card mt-4" aria-label="Current version">
            <h5 class="text-sm font-semibold text-gray-700">Current version</h5>
            {#if history.title}<p class="mt-2 font-semibold">{history.title}</p>{/if}
            <pre class="mt-2 text-sm break-words whitespace-pre-wrap">{history.description}</pre>
        </section>

        {#if history.revisions.length === 0}
            <p class="mt-4 text-gray-500">This post has never been edited.</p>
        {/if}
        {#each history.revisions as r, i (r.id)}
            <section class="card mt-4" aria-label="Earlier version">
                <h5 class="text-sm font-semibold text-gray-700">
                    Version {history.revisions.length - i}
                    <span class="font-normal text-gray-500">
                        · replaced {when(r.cat)}{#if r.editor}
                            by <a class="link" href="/users/{r.edited_by_id}/{r.editor}">{r.editor}</a>{/if}
                    </span>
                </h5>
                {#if r.title}<p class="mt-2 font-semibold">{r.title}</p>{/if}
                {#if r.tags}<p class="mt-1 text-xs text-gray-500">Tags: {r.tags}</p>{/if}
                <pre class="mt-2 text-sm break-words whitespace-pre-wrap">{r.description}</pre>
            </section>
        {/each}
    {/if}
</div>
