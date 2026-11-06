<script lang="ts">
    import { onMount } from 'svelte';
    import * as api from '#lib/api.ts';
    import Icon from '#lib/components/Icon.svelte';
    import Pagination from '#lib/components/Pagination.svelte';
    import { markdownToText } from '#lib/markdown.ts';
    import type { Tag } from '#lib/types.ts';

    const PER_PAGE = 20;
    let tags = $state<Tag[]>([]);
    let count = $state(0);
    let page = $state(1);
    let searchQuery = $state('');
    let pages = $derived(Math.ceil(count / PER_PAGE));

    function show(list: Tag[]) {
        tags = list.map((t) => ({ ...t, info: markdownToText(t.info || '') }));
    }

    async function fetchTags(after: string, back = false) {
        const res = await api.post('tags', { tag: after, ...(back ? { direction: 'back' } : {}) });
        if (res.status !== 200) return;
        const j = await api.json<{ data: Tag[]; count: number }>(res);
        show(j.data);
        count = j.count;
    }

    const first = () => ((page = 1), fetchTags(''));
    const next = () => tags.length && ((page += 1), fetchTags(tags[tags.length - 1].name));
    const prev = () => tags.length && ((page -= 1), page <= 1 ? first() : fetchTags(tags[0].name, true));

    async function search() {
        if (!searchQuery) return first();
        const res = await api.post('search-tags', { tag: searchQuery, tags_per_page: PER_PAGE });
        if (res.status === 200) show((await api.json<{ data: Tag[] }>(res)).data);
    }

    onMount(first);
</script>

<svelte:head>
    <title>Tags ❤ Kunjika</title>
</svelte:head>

<div class="mt-5">
    <h4 class="text-xl font-bold">Tags</h4>
    <p class="mt-2 text-gray-700">
        A tag is a keyword or label which is used to categorize similar questions. Tags make it easier to
        find similar questions.
    </p>
    <div class="relative mt-4 max-w-sm">
        <Icon name="search" size={16} class="absolute top-2.5 left-3 text-gray-400" />
        <input type="text" id="search" class="input pl-9" placeholder="Filter by tag" bind:value={searchQuery} onkeyup={search} />
    </div>
    <div class="mt-5 grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-4">
        {#each tags as t (t.name)}
            <div class="card flex flex-col gap-2">
                <a class="font-medium text-brand-dark" href="/questions/tagged/{encodeURIComponent(t.name)}">{t.name}</a>
                <p class="text-xs text-gray-600">{t.info ? `${t.info.slice(0, 140)} ...` : ''}</p>
                <span class="mt-auto text-[11px] text-gray-500"
                    >{t.post_count || 0} questions, {t.daily_count || 0} today, {t.weekly_count || 0} this week</span
                >
            </div>
        {/each}
    </div>
    {#if !searchQuery && pages > 1}
        <Pagination {page} {pages} onfirst={first} onprev={prev} onnext={next} />
    {/if}
</div>
