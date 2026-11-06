<script lang="ts">
    import { goto } from '$app/navigation';
    import { page } from '$app/state';
    import { onMount } from 'svelte';
    import * as api from '#lib/api.ts';
    import Editor from '#lib/components/Editor.svelte';
    import TagInput from '#lib/components/TagInput.svelte';
    import { firstDuplicateTag } from '#lib/tags.ts';
    import type { EditablePost } from '#lib/types.ts';

    let id = $derived(page.params.id ?? '');
    let title = $state('');
    let body = $state('');
    let tags = $state<string[]>([]);
    let loaded = $state(false);
    let notAuthor = $state(false);
    let formError = $state('');
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
        title = j.data.title;
        body = j.data.description;
        tags = j.data.tags ?? [];
        // Only the author may edit; the API enforces this too.
        notAuthor = !page.data.user || j.data.posted_by_id !== page.data.user.id;
        loaded = true;
    });

    async function submit(e: SubmitEvent) {
        e.preventDefault();
        if (body.length < 20 || body.length > 100000) {
            formError = 'Question length should be 20 to 1000000 Characters!';
            return;
        }
        if (tags.length < 1) {
            formError = 'At least one tag should be supplied.';
            return;
        }
        if (tags.length > 5) {
            formError = 'Too many tags.';
            return;
        }
        const duplicate = firstDuplicateTag(tags);
        if (duplicate) {
            formError = `Duplicate tag: ${duplicate}`;
            return;
        }
        formError = '';
        submitting = true;
        const res = await api.post(`edit-post/${id}`, { title, description: body, tag_list: tags });
        const j = await api.json<{ data?: { id: number; slug: string }; message?: string }>(res);
        submitting = false;
        if (res.status === 200 && j.data) {
            await goto(`/questions/${j.data.id}/${j.data.slug}`);
        } else {
            formError = j.message || 'Something went wrong, try again!';
        }
    }
</script>

<svelte:head>
    <title>Edit question ❤ Kunjika</title>
</svelte:head>

<div class="mt-5">
    {#if formError}<div class="alert-danger mb-4" role="alert" id="editError">{formError}</div>{/if}
    {#if notAuthor}
        <div class="alert-warning mb-4" role="alert" id="editNotAuthor">
            Only the author of a post can edit it.
        </div>
    {/if}
    <h4 class="text-xl font-normal">Edit your question</h4>
    <hr class="my-3 border-gray-200" />
    {#if loaded && !notAuthor}
        <form onsubmit={submit} class="space-y-4">
            <div>
                <label for="title" class="label">Summary</label>
                <input bind:value={title} class="input" id="title" type="text" minlength="6" maxlength="256" required />
            </div>
            <div>
                <span class="label">Question</span>
                <Editor bind:value={body} />
            </div>
            <div>
                <label for="tags" class="label">Tags</label>
                <TagInput bind:tags />
            </div>
            <div class="flex justify-center">
                <button type="submit" class="btn-primary" disabled={submitting}>Edit</button>
            </div>
        </form>
    {/if}
</div>
