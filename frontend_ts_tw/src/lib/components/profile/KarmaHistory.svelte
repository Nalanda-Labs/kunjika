<script lang="ts">
    import { onMount } from 'svelte';
    import * as api from '#lib/api.ts';
    import { timeAgo } from '#lib/time.ts';
    import type { KarmaEvent } from '#lib/types.ts';

    // Where a user's karma came from: one row per change, newest first.
    let { id }: { id: string | number } = $props();

    const PER_PAGE = 20;
    let events = $state<KarmaEvent[]>([]);
    let karma = $state<number | null>(null);
    // Cursor pagination: null once the history has been read to the end.
    let nextCursor = $state<string | null>(null);
    let loaded = $state(false);
    let loading = $state(false);
    let error = $state('');

    async function load(more = false) {
        loading = true;
        error = '';
        try {
            const res = await api.post(`users/${id}/karma/`, {
                per_page: PER_PAGE,
                ...(more && nextCursor ? { cursor: nextCursor } : {})
            });
            const j = await api.json<{
                data?: KarmaEvent[];
                next_cursor?: string | null;
                karma?: number;
                message?: string;
            }>(res);
            if (res.status === 200) {
                const rows = j.data ?? [];
                const seen = new Set(events.map((e) => e.id));
                events = more ? [...events, ...rows.filter((e) => !seen.has(e.id))] : rows;
                nextCursor = j.next_cursor ?? null;
                karma = j.karma ?? null;
            } else {
                error = j.message || 'Could not load the karma history.';
            }
        } catch {
            error = 'Could not load the karma history. Please try again.';
        }
        loaded = true;
        loading = false;
    }

    onMount(() => load());

    const TEXT: Record<KarmaEvent['reason'], string> = {
        vote_up: 'Vote up on',
        vote_down: 'Vote down on',
        accepted: 'Answer accepted in',
        unaccepted: 'Answer no longer accepted in',
        post_deleted: 'Post deleted:',
        opening_balance: 'Karma earned before history was kept'
    };

    function href(e: KarmaEvent): string {
        const base = `/questions/${e.question_id}/${e.question_slug ?? ''}`;
        return e.is_answer ? `${base}#${e.post_id}` : base;
    }
</script>

<div class="mt-3">
    {#if error}<div class="alert-danger" role="alert">{error}</div>{/if}
    {#if karma !== null}
        <p class="text-sm text-gray-600">Karma: <strong id="karmaTotal">{karma}</strong></p>
    {/if}
    {#if loaded && events.length === 0 && !error}
        <p class="py-6 text-gray-500">No karma changes yet.</p>
    {/if}
    <ul aria-label="Karma history">
        {#each events as e (e.id)}
            <li class="flex items-baseline gap-3 border-b border-gray-200 py-3 text-sm">
                <span
                    class="w-12 shrink-0 text-right font-semibold {e.delta >= 0 ? 'text-green-700' : 'text-red-700'}"
                    >{e.delta > 0 ? '+' : ''}{e.delta}</span
                >
                <span class="min-w-0 flex-1">
                    {TEXT[e.reason] ?? e.reason}
                    {#if e.reason !== 'opening_balance'}
                        {#if e.question_title}
                            <a class="link" href={href(e)}>{e.question_title}</a>
                        {:else if e.reason !== 'post_deleted'}
                            <span class="text-gray-500">a post that has since been deleted</span>
                        {:else}
                            <span class="text-gray-500">the karma it had earned was taken back</span>
                        {/if}
                    {/if}
                </span>
                <span class="shrink-0 text-xs text-gray-500">{timeAgo(e.cat)}</span>
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
</div>
