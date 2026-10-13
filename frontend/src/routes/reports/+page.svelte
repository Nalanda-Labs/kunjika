<script>
    import * as api from '$lib/api.js';
    import { page } from '$app/stores';
    import { browser } from '$app/environment';
    import getCookie from '../../lib/cookie.js';

    const STATUSES = ['open', 'reviewed', 'dismissed', 'actioned'];

    $: isReviewer =
        $page.data.user &&
        ($page.data.user.is_moderator || $page.data.user.is_superuser);

    let status = 'open';
    let reports = [];
    let count = 0;
    let loading = false;
    let error = '';
    let notice = '';
    let reviewing = null;

    async function load() {
        if (!isReviewer) {
            return;
        }
        loading = true;
        error = '';
        try {
            const response = await api.post('reports/', { status, per_page: 50 });
            const j = JSON.parse(await response.text());
            if (response.status === 200) {
                reports = j.data.reports;
                count = j.count;
            } else if (response.status === 403) {
                error = 'Only moderators can view reports.';
            } else {
                error = j.message || 'Could not load reports.';
            }
        } catch (e) {
            error = 'Could not load reports. Please try again.';
        }
        loading = false;
    }

    $: if (isReviewer && browser) {
        status;
        load();
    }

    function formatTs(ts) {
        const d = new Date(typeof ts === 'number' ? ts * 1000 : ts);
        return isNaN(d) ? String(ts) : d.toLocaleString();
    }

    async function review(id, newStatus) {
        if (reviewing) {
            return;
        }
        const confirmMsg =
            newStatus === 'actioned'
                ? 'This will DELETE the reported post. Continue?'
                : `Mark report #${id} as ${newStatus}?`;
        if (!confirm(confirmMsg)) {
            return;
        }
        reviewing = id;
        notice = '';
        error = '';
        try {
            const xsrf_token = browser ? getCookie('xsrf_token') : '';
            const response = await api.post(`reports/${id}/review/`, { status: newStatus }, xsrf_token);
            const j = JSON.parse(await response.text());
            if (response.status === 200 && j.success) {
                notice = `Report #${id} marked as ${newStatus}.`;
                await load();
            } else {
                error = j.message || j.data || 'Could not review the report.';
            }
        } catch (e) {
            error = 'Could not review the report. Please try again.';
        }
        reviewing = null;
    }
</script>

<svelte:head>
    <title>Reports — moderation queue</title>
</svelte:head>

{#if !isReviewer}
    <div class="alert alert-danger" role="alert" style="margin-top:20px">
        Only moderators can view this page.
    </div>
{:else}
    <h3 style="margin-top:20px">Reports <small class="text-muted">moderation queue</small></h3>

    {#if notice}
        <div class="alert alert-success" role="alert">{notice}</div>
    {/if}
    {#if error}
        <div class="alert alert-danger" role="alert">{error}</div>
    {/if}

    <div class="btn-group" role="group" aria-label="Filter by status" style="margin:10px 0 20px">
        {#each STATUSES as s}
            <button
                type="button"
                class="btn {s === status ? 'btn-primary' : 'btn-outline-primary'}"
                on:click={() => (status = s)}
            >
                {s}
            </button>
        {/each}
    </div>

    {#if loading}
        <p>Loading reports…</p>
    {:else if reports.length === 0}
        <p class="text-muted">No {status} reports.</p>
    {:else}
        <p class="text-muted">{count} {status} report{count === 1 ? '' : 's'}</p>
        {#each reports as r}
            <div class="card" style="margin-bottom:15px">
                <div class="card-body">
                    <h5 class="card-title">
                        <span class="badge bg-warning text-dark" style="margin-right:8px"
                            >{r.reason}</span
                        >
                        {#if r.post_title}
                            <a href="/questions/{r.post_id}">{r.post_title}</a>
                        {:else if r.post_excerpt}
                            <a href="/questions/{r.post_id}">Post #{r.post_id}</a>
                        {:else}
                            <span class="text-muted">(post deleted) #{r.post_id}</span>
                        {/if}
                        {#if r.open_count > 1}
                            <span
                                class="badge bg-danger"
                                style="margin-left:8px"
                                title="{r.open_count} open reports on this post"
                                >{r.open_count} reports</span
                            >
                        {/if}
                    </h5>
                    <h6 class="card-subtitle mb-2 text-muted">
                        #{r.id} · reported by
                        <a href="/users/{r.reported_by_id}/{r.reporter}">{r.reporter}</a>
                        {#if r.author}
                            · author
                            <a href="/users/{r.author_id}/{r.author}">{r.author}</a>
                        {/if}
                        · {r.is_question === false ? 'answer' : 'question'}
                        · {formatTs(r.created_at)}
                    </h6>
                    {#if r.post_excerpt}
                        <p class="card-text text-muted">{r.post_excerpt}…</p>
                    {/if}
                    {#if r.details}
                        <p class="card-text">
                            <strong>Details:</strong>
                            {r.details}
                        </p>
                    {/if}
                    {#if status === 'open'}
                        <button
                            class="btn btn-sm btn-outline-secondary"
                            style="margin-right:5px"
                            disabled={reviewing === r.id}
                            on:click={() => review(r.id, 'reviewed')}
                            >Mark reviewed</button
                        >
                        <button
                            class="btn btn-sm btn-outline-secondary"
                            style="margin-right:5px"
                            disabled={reviewing === r.id}
                            on:click={() => review(r.id, 'dismissed')}>Dismiss</button
                        >
                        <button
                            class="btn btn-sm btn-danger"
                            disabled={reviewing === r.id}
                            on:click={() => review(r.id, 'actioned')}
                            title="Delete the reported post"
                            >Delete post</button
                        >
                    {:else}
                        <span class="text-muted small">Status: {r.status}</span>
                    {/if}
                </div>
            </div>
        {/each}
    {/if}
{/if}

