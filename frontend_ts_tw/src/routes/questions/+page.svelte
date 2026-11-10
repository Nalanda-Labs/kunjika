<script lang="ts">
    import { page } from '$app/state';
    import PagedQuestions from '#lib/components/PagedQuestions.svelte';
    import { api_base } from '#lib/config.ts';
    import { fetchQuestions } from '#lib/questions.ts';

    const PER_PAGE = 30;
    let tab = $state<'all' | 'unanswered'>('all');
    let allCount = $state(0);
    let unansweredCount = $state(0);
    let unansweredOpened = $state(false); // load the tab on first visit, then keep it
    let count = $derived(tab === 'all' ? allCount : unansweredCount);

    const loadAll = (uat: string, back: boolean) =>
        fetchQuestions('questions/', {
            uat,
            questions_per_page: PER_PAGE,
            ...(back ? { direction: 'back' } : {})
        });
    const loadUnanswered = (uat: string, back: boolean) =>
        fetchQuestions('unanswered-questions/', {
            uat,
            questions_per_page: PER_PAGE,
            ...(back ? { direction: 'back' } : {})
        });

    function select(e: MouseEvent, t: 'all' | 'unanswered') {
        e.preventDefault();
        tab = t;
        if (t === 'unanswered') unansweredOpened = true;
    }
</script>

<svelte:head>
    <title>All questions ❤ Kunjika</title>
    <link rel="alternate" type="application/rss+xml" title="Newest questions" href="{api_base}/feeds/questions/" />
</svelte:head>

<div class="mt-5">
    <h4 class="flex items-center justify-between text-xl font-bold">
        <span>All questions ({count} questions)</span>
        {#if page.data.user}
            <a href="/ask" class="btn-primary">Ask</a>
        {/if}
    </h4>

    <div class="mt-5 flex border-b border-gray-200">
        <a href="#all-questions" class="tab-link" class:active={tab === 'all'} onclick={(e) => select(e, 'all')}
            >All Questions</a
        >
        <a
            href="#unanswered"
            class="tab-link"
            class:active={tab === 'unanswered'}
            onclick={(e) => select(e, 'unanswered')}>Unanswered</a
        >
    </div>

    <div id="all-questions" hidden={tab !== 'all'}>
        <PagedQuestions load={loadAll} perPage={PER_PAGE} bind:count={allCount} />
    </div>
    {#if unansweredOpened}
        <div id="unanswered" hidden={tab !== 'unanswered'}>
            <PagedQuestions
                load={loadUnanswered}
                perPage={PER_PAGE}
                bind:count={unansweredCount}
                empty="Every question has an answer."
            />
        </div>
    {/if}
</div>
