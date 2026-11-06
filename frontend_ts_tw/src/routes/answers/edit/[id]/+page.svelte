<script lang="ts">
    import { goto } from '$app/navigation';
    import { page } from '$app/state';
    import { onMount } from 'svelte';
    import * as api from '#lib/api.ts';
    import Editor from '#lib/components/Editor.svelte';
    import type { EditablePost } from '#lib/types.ts';

    let id = $derived(page.params.id ?? '');
    let body = $state('');
    let loaded = $state(false);
    let notAuthor = $state(false);
    let formError = $state('');
    let editSuccess = $state(false);
    let submitting = $state(false);

    onMount(async () => {
        const res = await api.get(`edit/question/${id}`);
        if (res.status === 404) {
            goto('/404');
            return;
        }
        const j = await api.json<{ data?: EditablePost; message?: string }>(res);
        if (res.status !== 200 || !j.data) {
            formError = j.message || 'Could not load the post.';
            return;
        }
        body = j.data.description;
        notAuthor = !page.data.user || j.data.posted_by_id !== page.data.user.id;
        loaded = true;
    });

    async function submit(e: SubmitEvent) {
        e.preventDefault();
        if (body.length < 20 || body.length > 100000) {
            formError = 'Answer length should be 20 to 100000 characters!';
            return;
        }
        formError = '';
        editSuccess = false;
        submitting = true;
        const res = await api.post(`edit/answer/${id}`, { description: body });
        const j = await api.json<{ message?: string }>(res);
        submitting = false;
        if (res.status === 200) editSuccess = true;
        else formError = j.message || 'Something went wrong, try again!';
    }
</script>

<svelte:head>
    <title>Edit answer ❤ Kunjika</title>
</svelte:head>

<div class="mt-5">
    {#if formError}<div class="alert-danger mb-4" role="alert" id="editError">{formError}</div>{/if}
    {#if notAuthor}
        <div class="alert-warning mb-4" role="alert" id="editNotAuthor">
            Only the author of an answer can edit it.
        </div>
    {/if}
    {#if editSuccess}<div class="alert-success mb-4" role="alert" id="editSuccess">The edit is successful.</div>{/if}
    <h4 class="text-xl font-normal">Edit your answer</h4>
    <hr class="my-3 border-gray-200" />
    {#if loaded && !notAuthor}
        <form onsubmit={submit} class="space-y-4">
            <Editor bind:value={body} />
            <div class="flex justify-center">
                <button type="submit" class="btn-primary" disabled={submitting}>Edit</button>
            </div>
        </form>
    {/if}
</div>
