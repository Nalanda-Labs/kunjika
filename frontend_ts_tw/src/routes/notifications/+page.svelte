<script lang="ts">
    import { browser } from '$app/env';
    import { page } from '$app/state';
    import * as api from '#lib/api.ts';
    import { unread } from '#lib/notifications.svelte.ts';
    import { timeAgo } from '#lib/time.ts';
    import type { Notification } from '#lib/types.ts';

    const PER_PAGE = 20;

    let items = $state<Notification[]>([]);
    // Cursor pagination: null once the list has been read to the end.
    let nextCursor = $state<string | null>(null);
    let loaded = $state(false);
    let loadingMore = $state(false);
    let error = $state('');
    let emailEnabled = $state<boolean | null>(null);
    let followed = $state<string[]>([]);

    async function load(more = false) {
        if (more) loadingMore = true;
        error = '';
        try {
            const res = await api.post('notifications/', {
                per_page: PER_PAGE,
                ...(more && nextCursor ? { cursor: nextCursor } : {})
            });
            const j = await api.json<{
                data?: Notification[];
                next_cursor?: string | null;
                unread?: number;
                message?: string;
            }>(res);
            if (res.status === 200) {
                const rows = j.data ?? [];
                const seen = new Set(items.map((n) => n.id));
                items = more ? [...items, ...rows.filter((n) => !seen.has(n.id))] : rows;
                nextCursor = j.next_cursor ?? null;
                unread.count = j.unread ?? 0;
            } else {
                error = j.message || 'Could not load notifications.';
            }
        } catch {
            error = 'Could not load notifications. Please try again.';
        }
        loaded = true;
        loadingMore = false;
    }

    async function loadSettings() {
        const [settings, tags] = await Promise.all([
            api.post('notifications/settings/', {}),
            api.post('tags/followed/', {})
        ]);
        if (settings.status === 200) {
            emailEnabled = (await api.json<{ email_notifications?: boolean }>(settings)).email_notifications ?? true;
        }
        if (tags.status === 200) followed = (await api.json<{ data?: string[] }>(tags)).data ?? [];
    }

    $effect(() => {
        if (!browser || !page.data.user) return;
        load();
        loadSettings().catch(() => {});
    });

    /** Mark one notification read, or all of them when `id` is omitted. */
    async function markRead(id?: number) {
        const res = await api.post('notifications/read/', id === undefined ? {} : { id });
        if (res.status !== 200) return;
        unread.count = (await api.json<{ unread?: number }>(res)).unread ?? 0;
        for (const n of items) if (id === undefined || n.id === id) n.read = true;
    }

    async function setEmail(enabled: boolean) {
        const res = await api.post('notifications/settings/', { email_notifications: enabled });
        if (res.status === 200) {
            emailEnabled = (await api.json<{ email_notifications?: boolean }>(res)).email_notifications ?? enabled;
        } else {
            alert('Could not save the setting.');
        }
    }

    async function unfollow(tag: string) {
        const res = await api.post(`tags/follow/${encodeURIComponent(tag)}`, { following: false });
        if (res.status === 200) followed = followed.filter((t) => t !== tag);
    }

    const TEXT: Record<Notification['kind'], string> = {
        answer: 'answered',
        reply: 'replied to you in',
        accepted: 'accepted your answer to',
        upvote: 'upvoted your post in',
        tag_question: 'asked a question in a tag you follow:'
    };

    function href(n: Notification): string {
        const base = `/questions/${n.question_id}/${n.question_slug ?? ''}`;
        // Link to the answer itself; the question page finds the page it is on.
        return n.post_id && n.post_id !== n.question_id ? `${base}#${n.post_id}` : base;
    }
</script>

<svelte:head>
    <title>Notifications ❤ Kunjika</title>
</svelte:head>

{#if !page.data.user}
    <div class="alert-danger mt-5" role="alert">Please <a class="link" href="/login">log in</a> to see your notifications.</div>
{:else}
    <div class="mt-5">
        <h3 class="flex flex-wrap items-center justify-between gap-2 text-2xl font-normal">
            Notifications
            {#if items.some((n) => !n.read)}
                <button type="button" class="btn-secondary text-sm" onclick={() => markRead()}>Mark all as read</button>
            {/if}
        </h3>
        {#if error}<div class="alert-danger mt-3" role="alert">{error}</div>{/if}

        {#if loaded && items.length === 0 && !error}
            <p class="mt-4 text-gray-500">Nothing yet. You will be told here when someone answers, replies, upvotes or accepts.</p>
        {/if}
        <ul class="mt-3" aria-label="Notifications">
            {#each items as n (n.id)}
                <li class="flex items-start gap-3 border-b border-gray-200 py-3 {n.read ? '' : 'bg-sky-50'} px-2">
                    <span
                        class="mt-2 h-2 w-2 shrink-0 rounded-full {n.read ? 'bg-transparent' : 'bg-brand'}"
                        title={n.read ? '' : 'Unread'}
                    ></span>
                    <div class="min-w-0 flex-1 text-sm">
                        {#if n.actor}<a class="link" href="/users/{n.actor_id}/{n.actor}">{n.actor}</a>{:else}Someone{/if}
                        {TEXT[n.kind] ?? 'did something in'}
                        {#if n.question_title}
                            <a class="link" href={href(n)} onclick={() => !n.read && markRead(n.id)}>{n.question_title}</a>
                        {:else}
                            <span class="text-gray-500">a question that has since been deleted</span>
                        {/if}
                        <span class="block text-xs text-gray-500">{timeAgo(n.cat)}</span>
                    </div>
                </li>
            {/each}
        </ul>
        {#if nextCursor}
            <div class="mt-4 flex justify-center">
                <button type="button" class="btn-secondary" disabled={loadingMore} onclick={() => load(true)}
                    >{loadingMore ? 'Loading…' : 'Load more'}</button
                >
            </div>
        {/if}

        <h4 class="mt-8 text-lg font-semibold">Settings</h4>
        {#if emailEnabled !== null}
            <label class="mt-2 flex items-center gap-2 text-sm">
                <input
                    type="checkbox"
                    checked={emailEnabled}
                    onchange={(e) => setEmail(e.currentTarget.checked)}
                />
                Email me when a question I took part in gets a new answer
            </label>
        {/if}

        <h4 class="mt-6 text-lg font-semibold">Followed tags</h4>
        {#if followed.length === 0}
            <p class="mt-2 text-sm text-gray-500">
                You are not following any tags. Open a tag from <a class="link" href="/tags">Tags</a> and press Follow to hear
                about new questions in it.
            </p>
        {:else}
            <ul class="mt-2 flex flex-wrap gap-2">
                {#each followed as t (t)}
                    <li class="inline-flex items-center gap-2 rounded bg-gray-100 px-2 py-1 text-sm">
                        <a class="link" href="/questions/tagged/{encodeURIComponent(t)}">{t}</a>
                        <button type="button" class="text-gray-500 hover:text-danger" aria-label="Unfollow {t}" onclick={() => unfollow(t)}
                            >×</button
                        >
                    </li>
                {/each}
            </ul>
        {/if}
    </div>
{/if}
