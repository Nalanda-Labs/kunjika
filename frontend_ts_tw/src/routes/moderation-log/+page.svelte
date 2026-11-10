<script lang="ts">
    import { browser } from '$app/env';
    import { page } from '$app/state';
    import * as api from '#lib/api.ts';
    import { isReviewer, type ModLogEntry } from '#lib/types.ts';

    // Who deleted, closed, reopened, suspended or reinstated what.
    const PER_PAGE = 30;

    let reviewer = $derived(isReviewer(page.data.user));
    let entries = $state<ModLogEntry[]>([]);
    let nextCursor = $state<string | null>(null);
    let loaded = $state(false);
    let loading = $state(false);
    let error = $state('');

    async function load(more = false) {
        loading = true;
        error = '';
        try {
            const res = await api.post('moderation/log/', {
                per_page: PER_PAGE,
                ...(more && nextCursor ? { cursor: nextCursor } : {})
            });
            const j = await api.json<{ data?: ModLogEntry[]; next_cursor?: string | null; message?: string }>(res);
            if (res.status === 200) {
                const rows = j.data ?? [];
                const seen = new Set(entries.map((e) => e.id));
                entries = more ? [...entries, ...rows.filter((e) => !seen.has(e.id))] : rows;
                nextCursor = j.next_cursor ?? null;
            } else {
                error = j.message || 'Could not load the moderation log.';
            }
        } catch {
            error = 'Could not load the moderation log. Please try again.';
        }
        loaded = true;
        loading = false;
    }

    $effect(() => {
        if (browser && reviewer) load();
    });

    const VERB: Record<ModLogEntry['action'], string> = {
        delete_post: 'deleted',
        close_question: 'closed',
        reopen_question: 'reopened',
        suspend_user: 'suspended',
        reinstate_user: 'reinstated'
    };
    const when = (secs: number) => new Date(secs * 1000).toLocaleString();
</script>

<svelte:head>
    <title>Moderation log ❤ Kunjika</title>
</svelte:head>

{#if !reviewer}
    <div class="alert-danger mt-5" role="alert">Only moderators can view this page.</div>
{:else}
    <h3 class="mt-5 text-2xl font-normal">Moderation log</h3>
    {#if error}<div class="alert-danger mt-3" role="alert">{error}</div>{/if}
    {#if loaded && entries.length === 0 && !error}
        <p class="mt-4 text-gray-500">Nothing has been logged yet.</p>
    {/if}
    <ul class="mt-3" aria-label="Moderation log">
        {#each entries as e (e.id)}
            <li class="border-b border-gray-200 py-3 text-sm">
                {#if e.actor}<a class="link" href="/users/{e.actor_id}/{e.actor}">{e.actor}</a>{:else}A deleted user{/if}
                {VERB[e.action] ?? e.action}
                {#if e.target_user && e.action !== 'delete_post'}
                    <a class="link" href="/users/{e.target_user_id}/{e.target_user}">{e.details || e.target_user}</a>
                {:else if e.question_id}
                    <a class="link" href="/questions/{e.question_id}/{e.question_slug ?? ''}">{e.details}</a>
                {:else}
                    {e.details}
                {/if}
                <span class="block text-xs text-gray-500">{when(e.cat)}</span>
            </li>
        {/each}
    </ul>
    {#if nextCursor}
        <div class="mt-4 flex justify-center">
            <button type="button" class="btn-secondary" disabled={loading} onclick={() => load(true)}
                >{loading ? 'Loading…' : 'Load more'}</button
            >
        </div>
    {/if}
{/if}
