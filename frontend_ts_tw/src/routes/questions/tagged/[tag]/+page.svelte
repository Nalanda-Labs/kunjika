<script lang="ts">
    import { browser } from '$app/env';
    import { page } from '$app/state';
    import * as api from '#lib/api.ts';
    import Icon from '#lib/components/Icon.svelte';
    import { api_base } from '#lib/config.ts';
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

    let feed = $derived(`${api_base}/feeds/tagged/${encodeURIComponent(tag)}/`);

    // Following a tag puts its new questions in your notifications.
    let following = $state(false);
    $effect(() => {
        const current = tag;
        following = false;
        if (!browser || !page.data.user) return;
        api.post('tags/followed/', {})
            .then(async (res) => {
                if (res.status !== 200 || current !== tag) return;
                const names = (await api.json<{ data?: string[] }>(res)).data ?? [];
                following = names.some((n) => n.toLowerCase() === current.toLowerCase());
            })
            .catch(() => {});
    });

    async function toggleFollow() {
        const res = await api.post(`tags/follow/${encodeURIComponent(tag)}`, { following: !following });
        const j = await api.json<{ following?: boolean; message?: string }>(res);
        if (res.status === 200) following = !!j.following;
        else alert(j.message || 'Could not update the tag.');
    }
</script>

<svelte:head>
    <title>Questions tagged {tag} ❤ Kunjika</title>
    <link rel="alternate" type="application/rss+xml" title="Questions tagged {tag}" href={feed} />
</svelte:head>

<div class="mt-5">
    <h4 class="flex items-center justify-between text-xl font-bold">
        <span>Questions tagged {tag} ({count} questions)</span>
        {#if page.data.user}
            <a href="/ask" class="btn-primary">Ask</a>
        {/if}
    </h4>
    <div class="mt-3 flex flex-wrap items-center gap-2">
        <a class="btn-secondary" href="/tags/info/{encodeURIComponent(tag)}">Info</a>
        {#if page.data.user}
            <button type="button" class="btn-secondary" aria-pressed={following} onclick={toggleFollow}
                >{following ? 'Following' : 'Follow'}</button
            >
        {/if}
        <a class="action-link" href={feed} title="RSS feed of new questions in this tag"><Icon name="rss" size={16} />RSS</a>
    </div>
    {#key tag}
        <PagedQuestions {load} perPage={PER_PAGE} bind:count empty="No questions with this tag yet." />
    {/key}
</div>
