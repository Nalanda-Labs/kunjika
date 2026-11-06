<script lang="ts" module>
    import type { QuestionRow } from '#lib/types.ts';

    export interface QuestionsPage {
        questions: QuestionRow[];
        count: number;
        pinned?: QuestionRow[];
    }

    /**
     * Fetch one page. `uat` is the cursor ('' for the first page); `back`
     * asks for the page before it. Return null when there is nothing to show.
     */
    export type QuestionsLoader = (uat: string, back: boolean) => Promise<QuestionsPage | null>;
</script>

<script lang="ts">
    import { onMount } from 'svelte';
    import { cursor } from '#lib/time.ts';
    import Pagination from './Pagination.svelte';
    import QuestionRowView from './QuestionRow.svelte';

    let {
        load,
        perPage = 30,
        count = $bindable(0),
        empty = 'No questions yet.'
    }: { load: QuestionsLoader; perPage?: number; count?: number; empty?: string } = $props();

    let rows = $state<QuestionRow[]>([]);
    let pinned = $state<QuestionRow[]>([]);
    let page = $state(1);
    let loaded = $state(false);
    let pages = $derived(Math.ceil(count / perPage));

    async function fetchPage(uat: string, back: boolean) {
        try {
            const r = await load(uat, back);
            rows = r?.questions ?? [];
            count = r?.count ?? 0;
            if (r?.pinned) pinned = r.pinned;
        } catch (e) {
            alert(e instanceof Error ? e.message : String(e));
        }
        loaded = true;
    }

    function first() {
        page = 1;
        return fetchPage('', false);
    }
    function next() {
        if (!rows.length) return;
        page += 1;
        return fetchPage(cursor(rows[rows.length - 1].uat), false);
    }
    function prev() {
        if (!rows.length) return;
        page -= 1;
        if (page <= 1) return first();
        return fetchPage(cursor(rows[0].uat, true), true);
    }

    onMount(first);
</script>

<div>
    {#if page === 1}
        {#each pinned as q (q.id)}<QuestionRowView {q} pinned />{/each}
    {/if}
    {#each rows as q (q.id)}<QuestionRowView {q} />{/each}
    {#if loaded && !rows.length && !pinned.length}
        <p class="py-6 text-gray-500">{empty}</p>
    {/if}
    {#if pages > 1}
        <Pagination {page} {pages} onfirst={first} onprev={prev} onnext={next} />
    {/if}
</div>
