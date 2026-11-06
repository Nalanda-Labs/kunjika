<script lang="ts">
    import { onMount } from 'svelte';
    import * as api from '#lib/api.ts';
    import { cursor, timeAgo } from '#lib/time.ts';
    import type { UserPostRow } from '#lib/types.ts';
    import Pagination from '../Pagination.svelte';
    import TagChips from '../TagChips.svelte';

    // A user's answers or bookmarks: each row links to the question, at the answer.
    let {
        path,
        perPage = 30,
        extra = {},
        empty
    }: { path: string; perPage?: number; extra?: Record<string, unknown>; empty: string } = $props();

    let rows = $state<UserPostRow[]>([]);
    let count = $state(0);
    let page = $state(1);
    let loaded = $state(false);
    let pages = $derived(Math.ceil(count / perPage));

    async function fetchPage(uat: string, back = false) {
        const res = await api.post(path, { uat, ...extra, ...(back ? { direction: 'back' } : {}) });
        const j = await api.json<{ data?: UserPostRow[]; count?: number; message?: string }>(res);
        if (res.status === 200) {
            rows = j.data ?? [];
            count = j.count ?? 0;
        } else if (j.message === api.NO_ROWS) {
            rows = [];
            count = 0;
        } else {
            alert(j.message || 'Could not load the list.');
        }
        loaded = true;
    }

    const first = () => ((page = 1), fetchPage(''));
    const next = () => rows.length && ((page += 1), fetchPage(cursor(rows[rows.length - 1].uat)));
    const prev = () =>
        rows.length && ((page -= 1), page <= 1 ? first() : fetchPage(cursor(rows[0].uat, true), true));

    onMount(first);
</script>

<div class="mt-2">
    {#each rows as r (`${r.question_id}-${r.answer_id}`)}
        <div class="border-b border-gray-200 py-4">
            <a
                href="/questions/{r.question_id}/{r.slug}{r.answer_id ? `#${r.answer_id}` : ''}"
                class="text-base text-brand-dark hover:underline">{r.title}</a
            >
            <div class="mt-3 flex flex-wrap items-center justify-between gap-2">
                <TagChips tags={r.tags.split(',')} />
                <span class="text-xs text-gray-500">{timeAgo(r.uat, 'modified')}</span>
            </div>
        </div>
    {/each}
    {#if loaded && !rows.length}<p class="py-6 text-gray-500">{empty}</p>{/if}
    {#if pages > 1}<Pagination {page} {pages} onfirst={first} onprev={prev} onnext={next} />{/if}
</div>
