<script lang="ts">
    import { page } from '$app/state';
    import { onMount } from 'svelte';
    import * as api from '#lib/api.ts';
    import { diffLines, type DiffLine } from '#lib/diff.ts';
    import type { PostHistory } from '#lib/types.ts';

    // Edit history of one post: the current text, then each edit as a diff
    // against the version before it, newest first, then the original.
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

    interface Edit {
        key: number;
        number: number;
        when: number;
        editor: string | null;
        editorId: number | null;
        /** Old and new title when the edit changed it. */
        title: { from: string; to: string } | null;
        /** Old tag list, when the version before the edit recorded one. */
        tags: string | null;
        lines: DiffLine[];
        changed: boolean;
    }

    // `revisions[i]` is the text an edit replaced; what it became is the next
    // newer version: `revisions[i - 1]`, or the current post for the newest.
    let edits = $derived.by((): Edit[] => {
        if (!history) return [];
        const h = history;
        return h.revisions.map((r, i) => {
            const after = i === 0 ? h : h.revisions[i - 1];
            const lines = diffLines(r.description, after.description);
            const titleChanged = (r.title ?? '') !== (after.title ?? '');
            return {
                key: r.id,
                number: h.revisions.length - i,
                when: r.cat,
                editor: r.editor,
                editorId: r.edited_by_id,
                title: titleChanged ? { from: r.title ?? '', to: after.title ?? '' } : null,
                tags: r.tags,
                lines,
                changed: lines.some((l) => l.kind !== 'same')
            };
        });
    });
    let original = $derived(history?.revisions.at(-1) ?? null);

    const when = (secs: number) => new Date(secs * 1000).toLocaleString();
    const MARK = { same: ' ', add: '+', del: '-' } as const;
    const TINT = { same: '', add: 'bg-green-100 text-green-900', del: 'bg-red-100 text-red-900' } as const;
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

        {#each edits as e (e.key)}
            <section class="card mt-4" aria-label="Edit {e.number}">
                <h5 class="text-sm font-semibold text-gray-700">
                    Edit {e.number}
                    <span class="font-normal text-gray-500">
                        · {when(e.when)}{#if e.editor}
                            by <a class="link" href="/users/{e.editorId}/{e.editor}">{e.editor}</a>{/if}
                    </span>
                </h5>
                {#if e.title}
                    <p class="mt-2 text-sm">
                        Title: <del class="bg-red-100 text-red-900">{e.title.from}</del>
                        → <ins class="bg-green-100 text-green-900 no-underline">{e.title.to}</ins>
                    </p>
                {/if}
                {#if e.tags}<p class="mt-1 text-xs text-gray-500">Tags before this edit: {e.tags}</p>{/if}
                {#if e.changed}
                    <!-- One element per line: removed lines in red, added in green. -->
                    <div class="mt-2 overflow-x-auto font-mono text-sm" role="group" aria-label="Changes in edit {e.number}">
                        {#each e.lines as l, n (n)}
                            <div class="whitespace-pre-wrap {TINT[l.kind]}" data-diff={l.kind}><span
                                    class="inline-block w-4 text-gray-500 select-none"
                                    aria-hidden="true">{MARK[l.kind]}</span
                                >{#if l.kind === 'del'}<span class="sr-only">removed: </span>{:else if l.kind === 'add'}<span
                                        class="sr-only">added: </span
                                    >{/if}{l.text}</div>
                        {/each}
                    </div>
                {:else if !e.title}
                    <p class="mt-2 text-sm text-gray-500">No change to the text.</p>
                {/if}
            </section>
        {/each}

        {#if original}
            <section class="card mt-4" aria-label="Original version">
                <h5 class="text-sm font-semibold text-gray-700">Original version</h5>
                {#if original.title}<p class="mt-2 font-semibold">{original.title}</p>{/if}
                {#if original.tags}<p class="mt-1 text-xs text-gray-500">Tags: {original.tags}</p>{/if}
                <pre class="mt-2 text-sm break-words whitespace-pre-wrap">{original.description}</pre>
            </section>
        {/if}
    {/if}
</div>
