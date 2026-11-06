<script lang="ts">
    import * as api from '#lib/api.ts';
    import Modal from './Modal.svelte';

    const REASONS = [
        { value: 'spam', label: 'Spam — unsolicited promotion or advertising' },
        { value: 'abusive', label: 'Abusive — harassment, hate speech, or threats' },
        { value: 'off-topic', label: 'Off-topic — does not belong on this site' },
        { value: 'low-quality', label: 'Low quality — unclear, incomplete, or no effort' },
        { value: 'duplicate', label: 'Duplicate — already asked and answered' },
        { value: 'other', label: 'Other — explain in details below' }
    ];

    let isOpen = $state(false);
    let postId = $state<string | number | null>(null);
    let postTitle = $state('');
    let reason = $state('spam');
    let details = $state('');
    let submitting = $state(false);
    let error = $state('');
    let done = $state(false);

    export function open(id: string | number, title = '') {
        postId = id;
        postTitle = title;
        reason = 'spam';
        details = '';
        error = '';
        done = false;
        submitting = false;
        isOpen = true;
    }

    async function submit(e: SubmitEvent) {
        e.preventDefault();
        if (postId === null || submitting) return;
        submitting = true;
        error = '';
        try {
            const res = await api.post('report-post/', { post_id: Number(postId), reason, details });
            const j = await api.json<{ success?: boolean; data?: string; message?: string }>(res);
            if (res.status === 200 && j.success) {
                done = true;
            } else if (res.status === 429) {
                error = j.message || 'Report limit reached: at most 5 reports per day.';
            } else {
                error = j.data || j.message || 'Could not file the report.';
            }
        } catch {
            error = 'Could not file the report. Please try again.';
        }
        submitting = false;
    }
</script>

<Modal bind:open={isOpen} title="Report post" id="reportDialog">
    {#if done}
        <div class="alert-success" role="alert">
            Thanks — your report has been filed. A moderator will review it.
        </div>
        <div class="mt-4 flex justify-end">
            <button type="button" class="btn-primary" onclick={() => (isOpen = false)}>Close</button>
        </div>
    {:else}
        {#if postTitle}<p class="mb-3 text-sm text-gray-500">{postTitle}</p>{/if}
        <form onsubmit={submit} class="space-y-3">
            {#if error}<div class="alert-danger" role="alert">{error}</div>{/if}
            <div>
                <label class="label" for="reportReason">Reason</label>
                <select id="reportReason" class="input" bind:value={reason} disabled={submitting}>
                    {#each REASONS as r (r.value)}<option value={r.value}>{r.label}</option>{/each}
                </select>
            </div>
            <div>
                <label class="label" for="reportDetails">Details (optional)</label>
                <textarea
                    id="reportDetails"
                    class="input"
                    rows="3"
                    maxlength="2000"
                    placeholder="Anything a moderator should know…"
                    bind:value={details}
                    disabled={submitting}
                ></textarea>
            </div>
            <div class="flex justify-end gap-2">
                <button type="button" class="btn-secondary" onclick={() => (isOpen = false)} disabled={submitting}
                    >Cancel</button
                >
                <button type="submit" class="btn-danger" disabled={submitting}>
                    {submitting ? 'Reporting…' : 'Report'}
                </button>
            </div>
        </form>
    {/if}
</Modal>
