<script lang="ts">
    import { goto } from '$app/navigation';
    import { page } from '$app/state';
    import { onMount } from 'svelte';
    import * as api from '#lib/api.ts';
    import Editor from '#lib/components/Editor.svelte';

    const MIN = 100;
    const MAX = 1000000;
    let tag = $derived(page.params.tag ?? '');
    let info = $state('');
    let error = $state('');
    let saving = $state(false);

    onMount(async () => {
        const res = await api.get(`get-tag-info/${encodeURIComponent(tag)}`);
        const j = await api.json<{ info?: string; message?: string }>(res);
        if (res.status === 200) info = j.info ?? '';
        else error = j.message || 'Could not load the tag info.';
    });

    async function save(e: SubmitEvent) {
        e.preventDefault();
        if (info.length < MIN || info.length > MAX) {
            error = `Tag info must be ${MIN} to ${MAX} characters.`;
            return;
        }
        error = '';
        saving = true;
        const res = await api.post(`tags/edit/${encodeURIComponent(tag)}`, { tag_info: info });
        const j = await api.json<{ message?: string }>(res);
        saving = false;
        if (res.status === 200) await goto(`/tags/info/${encodeURIComponent(tag)}`);
        else error = j.message || 'Something went wrong, try again!';
    }
</script>

<svelte:head>
    <title>Edit Tag Info ❤ Kunjika</title>
</svelte:head>

<div class="mx-auto mt-5 max-w-3xl">
    <h4 class="text-xl font-bold">Edit info for {tag}</h4>
    <hr class="my-3 border-gray-200" />
    {#if error}<div class="alert-danger mb-4" role="alert">{error}</div>{/if}
    <form onsubmit={save} class="space-y-4">
        <Editor bind:value={info} minlength={MIN} maxlength={MAX} />
        <div class="flex justify-center">
            <button type="submit" class="btn-primary" disabled={saving}>Update</button>
        </div>
    </form>
</div>
