<script lang="ts">
    import { browser } from '$app/env';
    import { page } from '$app/state';
    import * as api from '#lib/api.ts';
    import { isReviewer, type Report } from '#lib/types.ts';

    const STATUSES = ['open', 'reviewed', 'dismissed', 'actioned'] as const;
    type Status = (typeof STATUSES)[number];

    let reviewer = $derived(isReviewer(page.data.user));
    let status = $state<Status>('open');
    let reports = $state<Report[]>([]);
    let count = $state(0);
    let loading = $state(false);
    let error = $state('');
    let notice = $state('');
    let reviewing = $state<number | null>(null);

    async function load() {
        loading = true;
        error = '';
        try {
            const res = await api.post('reports/', { status, per_page: 50 });
            const j = await api.json<{ data?: { reports: Report[] }; count?: number; message?: string }>(res);
            if (res.status === 200) {
                reports = j.data?.reports ?? [];
                count = j.count ?? 0;
            } else if (res.status === 403) {
                error = 'Only moderators can view reports.';
            } else {
                error = j.message || 'Could not load reports.';
            }
        } catch {
            error = 'Could not load reports. Please try again.';
        }
        loading = false;
    }

    $effect(() => {
        void status;
        if (browser && reviewer) load();
    });

    function formatTs(ts: string | number): string {
        const d = new Date(typeof ts === 'number' ? ts * 1000 : ts);
        return isNaN(d.getTime()) ? String(ts) : d.toLocaleString();
    }

    async function review(id: number, next: Exclude<Status, 'open'>) {
        if (reviewing) return;
        const msg =
            next === 'actioned' ? 'This will DELETE the reported post. Continue?' : `Mark report #${id} as ${next}?`;
        if (!confirm(msg)) return;
        reviewing = id;
        notice = '';
        error = '';
        try {
            const res = await api.post(`reports/${id}/review/`, { status: next });
            const j = await api.json<{ success?: boolean; message?: string; data?: string }>(res);
            if (res.status === 200 && j.success) {
                notice = `Report #${id} marked as ${next}.`;
                await load();
            } else {
                error = j.message || j.data || 'Could not review the report.';
            }
        } catch {
            error = 'Could not review the report. Please try again.';
        }
        reviewing = null;
    }
</script>

<svelte:head>
    <title>Reports — moderation queue</title>
</svelte:head>

{#if !reviewer}
    <div class="alert-danger mt-5" role="alert">Only moderators can view this page.</div>
{:else}
    <h3 class="mt-5 text-2xl font-normal">Reports <small class="text-base text-gray-500">moderation queue</small></h3>
    {#if notice}<div class="alert-success mt-3" role="alert">{notice}</div>{/if}
    {#if error}<div class="alert-danger mt-3" role="alert">{error}</div>{/if}

    <div class="my-5 inline-flex overflow-hidden rounded-md border border-gray-300" role="group" aria-label="Filter by status">
        {#each STATUSES as s (s)}
            <button
                type="button"
                class="px-4 py-1.5 text-sm capitalize {status === s ? 'bg-gray-800 text-white' : 'bg-white text-gray-700 hover:bg-gray-50'}"
                aria-pressed={status === s}
                onclick={() => (status = s)}>{s}</button
            >
        {/each}
    </div>

    {#if loading}
        <p>Loading reports…</p>
    {:else if reports.length === 0}
        <p class="text-gray-500">No {status} reports.</p>
    {:else}
        <p class="mb-3 text-gray-500">{count} {status} report{count === 1 ? '' : 's'}</p>
        {#each reports as r (r.id)}
            <div class="card mb-4">
                <h5 class="flex flex-wrap items-center gap-2 text-lg">
                    <span class="rounded bg-amber-300 px-2 py-0.5 text-xs font-medium text-gray-900">{r.reason}</span>
                    {#if r.post_title}
                        <a class="link" href="/questions/{r.post_id}/post">{r.post_title}</a>
                    {:else if r.post_excerpt}
                        <a class="link" href="/questions/{r.post_id}/post">Post #{r.post_id}</a>
                    {:else}
                        <span class="text-gray-500">(post deleted) #{r.post_id}</span>
                    {/if}
                    {#if (r.open_count ?? 0) > 1}
                        <span class="rounded-full bg-red-100 px-2 text-xs text-red-800" title="{r.open_count} open reports on this post"
                            >{r.open_count} reports</span
                        >
                    {/if}
                </h5>
                <p class="mt-1 text-sm text-gray-500">
                    #{r.id} · reported by <a class="link" href="/users/{r.reported_by_id}/{r.reporter}">{r.reporter}</a>
                    {#if r.author}· author <a class="link" href="/users/{r.author_id}/{r.author}">{r.author}</a>{/if}
                    · {r.is_question === false ? 'answer' : 'question'} · {formatTs(r.created_at)}
                </p>
                {#if r.post_excerpt}<p class="mt-2 text-sm text-gray-600">{r.post_excerpt}…</p>{/if}
                {#if r.details}<p class="mt-2 text-sm"><strong>Details:</strong> {r.details}</p>{/if}
                {#if status === 'open'}
                    <div class="mt-3 flex flex-wrap gap-2">
                        <button class="btn-secondary" disabled={reviewing === r.id} onclick={() => review(r.id, 'reviewed')}>Mark reviewed</button>
                        <button class="btn-secondary" disabled={reviewing === r.id} onclick={() => review(r.id, 'dismissed')}>Dismiss</button>
                        <button class="btn-danger" disabled={reviewing === r.id} onclick={() => review(r.id, 'actioned')}>Delete post</button>
                    </div>
                {:else}
                    <p class="mt-2 text-xs text-gray-500">Status: {r.status}</p>
                {/if}
            </div>
        {/each}
    {/if}
{/if}
