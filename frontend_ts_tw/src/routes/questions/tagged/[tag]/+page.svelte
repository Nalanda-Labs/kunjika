<script lang="ts">
    import { page } from '$app/state';
    import PagedQuestions from '#lib/components/PagedQuestions.svelte';
    import { fetchQuestions } from '#lib/questions.ts';

    const PER_PAGE = 30;
    let count = $state(0);
    let tag = $derived(page.params.tag ?? '');

    const load = (uat: string, back: boolean) =>
        fetchQuestions(`questions/tagged/${encodeURIComponent(tag)}`, {
            uat,
            questions_per_page: PER_PAGE,
            ...(back ? { direction: 'back' } : {})
        });
</script>

<svelte:head>
    <title>Questions tagged {tag} ❤ Kunjika</title>
</svelte:head>

<div class="mt-5">
    <h4 class="flex items-center justify-between text-xl font-bold">
        <span>Questions tagged {tag} ({count} questions)</span>
        {#if page.data.user}
            <a href="/ask" class="btn-primary">Ask</a>
        {/if}
    </h4>
    <a class="btn-secondary mt-3" href="/tags/info/{encodeURIComponent(tag)}">Info</a>
    {#key tag}
        <PagedQuestions {load} perPage={PER_PAGE} bind:count empty="No questions with this tag yet." />
    {/key}
</div>
