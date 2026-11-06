<script lang="ts">
    import { page } from '$app/state';
    import * as api from '#lib/api.ts';
    import QuestionRowView from '#lib/components/QuestionRow.svelte';
    import type { QuestionRow } from '#lib/types.ts';

    // The backend returns at most the 50 most relevant matches.
    const LIMIT = 50;

    let q = $derived(page.url.searchParams.get('q') ?? '');
    let questions = $state<QuestionRow[]>([]);
    let count = $state(0);
    let loading = $state(false);
    let requestId = 0;

    async function run(term: string) {
        const query = term.trim();
        const mine = ++requestId;
        if (!query) {
            questions = [];
            count = 0;
            return;
        }
        loading = true;
        try {
            const res = await api.post('search-questions/', { search: query, questions_per_page: LIMIT });
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
        run(q);
    });
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
</div>
