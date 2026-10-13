<script>
    import * as api from '$lib/api.js';
    import { browser } from '$app/environment';
    import getCookie from './cookie.js';

    export let post_id = null;
    export let post_title = '';

    const REASONS = [
        { value: 'spam', label: 'Spam — unsolicited promotion or advertising' },
        { value: 'abusive', label: 'Abusive — harassment, hate speech, or threats' },
        { value: 'off-topic', label: 'Off-topic — does not belong on this site' },
        { value: 'low-quality', label: 'Low quality — unclear, incomplete, or no effort' },
        { value: 'duplicate', label: 'Duplicate — already asked and answered' },
        { value: 'other', label: 'Other — explain in details below' }
    ];

    let reason = 'spam';
    let details = '';
    let submitting = false;
    let error = '';
    let done = false;

    export function open(id, title = '') {
        post_id = id;
        post_title = title;
        reason = 'spam';
        details = '';
        error = '';
        done = false;
        submitting = false;
        if (browser) {
            document.getElementById('reportDialog').style.display = 'block';
        }
    }

    function close() {
        if (browser) {
            document.getElementById('reportDialog').style.display = 'none';
        }
        post_id = null;
    }

    async function submit() {
        if (!post_id || submitting) {
            return;
        }
        submitting = true;
        error = '';
        try {
            const xsrf_token = browser ? getCookie('xsrf_token') : '';
            const response = await api.post(
                'report-post/',
                { post_id: parseInt(post_id), reason, details },
                xsrf_token
            );
            const j = JSON.parse(await response.text());
            if (response.status === 200 && j.success) {
                done = true;
            } else if (response.status === 200 && j.success === false) {
                error = j.data || 'Could not file the report.';
            } else if (response.status === 429) {
                error = j.message || 'Report limit reached: at most 5 reports per day.';
            } else {
                error = j.message || j.data || 'Could not file the report.';
            }
        } catch (e) {
            error = 'Could not file the report. Please try again.';
        }
        submitting = false;
    }
</script>

<div
    class="modal modal-dialog-centered"
    tabindex="-1"
    role="dialog"
    id="reportDialog"
    style="top:200px; display:none"
>
    <div class="modal-dialog" role="document">
        <div class="modal-content">
            <div class="modal-header">
                <h5 class="modal-title">Report post</h5>
            </div>
            <div class="modal-body">
                {#if done}
                    <div class="alert alert-success" role="alert">
                        Thanks — your report has been filed. A moderator will review it.
                    </div>
                    <div class="modal-footer">
                        <button type="button" class="btn btn-primary" on:click={close}
                            >Close</button
                        >
                    </div>
                {:else}
                    {#if post_title}
                        <p class="text-muted small">{post_title}</p>
                    {/if}
                    <form on:submit|preventDefault={submit}>
                        {#if error}
                            <div class="alert alert-danger" role="alert">{error}</div>
                        {/if}
                        <label class="form-label" for="reportReason">Reason</label>
                        <select
                            id="reportReason"
                            class="form-select"
                            bind:value={reason}
                            disabled={submitting}
                        >
                            {#each REASONS as r}
                                <option value={r.value}>{r.label}</option>
                            {/each}
                        </select>
                        <label class="form-label mt-3" for="reportDetails"
                            >Details (optional)</label
                        >
                        <textarea
                            id="reportDetails"
                            class="form-control"
                            rows="3"
                            maxlength="2000"
                            placeholder="Anything a moderator should know…"
                            bind:value={details}
                            disabled={submitting}
                        ></textarea>
                        <div class="modal-footer">
                            <button
                                type="button"
                                class="btn btn-secondary"
                                on:click={close}
                                disabled={submitting}>Cancel</button
                            >
                            <button
                                type="submit"
                                class="btn btn-danger"
                                disabled={submitting}
                            >
                                {#if submitting}Reporting…{:else}Report{/if}
                            </button>
                        </div>
                    </form>
                {/if}
            </div>
        </div>
    </div>
</div>
