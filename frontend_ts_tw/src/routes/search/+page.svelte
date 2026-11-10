<script lang="ts">
    import { goto } from '$app/navigation';
    import { page } from '$app/state';
    import * as api from '#lib/api.ts';
    import Pagination from '#lib/components/Pagination.svelte';
    import QuestionRowView from '#lib/components/QuestionRow.svelte';
    import type { QuestionRow } from '#lib/types.ts';

    // Results are ranked by relevance and paged with limit/offset; the page
    // number lives in the URL (`?q=…&page=N`) next to the query.
    const PER_PAGE = 20;

    let q = $derived(page.url.searchParams.get('q') ?? '');
    let pageNo = $derived.by(() => {
        const n = Number(page.url.searchParams.get('page'));
        return Number.isInteger(n) && n > 0 ? n : 1;
    });
    let questions = $state<QuestionRow[]>([]);
    let count = $state(0);
    let pages = $derived(Math.ceil(count / PER_PAGE));
    let loading = $state(false);
    let requestId = 0;

    async function run(term: string, wanted: number) {
        const query = term.trim();
        const mine = ++requestId;
        if (!query) {
            questions = [];
            count = 0;
            return;
        }
        loading = true;
        try {
            const res = await api.post('search-questions/', {
                search: query,
                questions_per_page: PER_PAGE,
                offset: (wanted - 1) * PER_PAGE
            });
            const j = await api.json<{ data?: { questions: QuestionRow[] }; count?: number }>(res);
            if (mine !== requestId) return;
            questions = (res.status === 200 && j.data?.questions) || [];
            count = res.status === 200 ? (j.count ?? questions.length) : 0;
        } catch {
            if (mine === requestId) {
                questions = [];
                count = 0;
            }
        } finally {
            if (mine === requestId) loading = false;
        }
    }

    $effect(() => {
        run(q, pageNo);
    });

    function goPage(n: number) {
        const u = new URL(page.url.href);
        if (n > 1) u.searchParams.set('page', String(n));
        else u.searchParams.delete('page');
        return goto(u);
    }
</script>

<svelte:head>
    <title>Search{q ? `: ${q}` : ''} ❤ Kunjika</title>
</svelte:head>

<div class="mt-5">
    <h4 class="text-xl font-bold">Search results for "{q}" ({count})</h4>
    {#if loading}<p class="mt-3">Searching…</p>{/if}
    {#if !loading && q.trim() !== '' && count === 0}
        <p class="mt-3 text-gray-600">No questions matched your search.</p>
    {/if}
    {#each questions as row (row.id)}<QuestionRowView q={row} />{/each}
    {#if pages > 1}
        <Pagination
            page={pageNo}
            {pages}
            onfirst={() => goPage(1)}
            onprev={() => goPage(pageNo - 1)}
            onnext={() => goPage(pageNo + 1)}
        />
    {/if}
</div>
