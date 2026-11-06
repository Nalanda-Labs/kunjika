<script lang="ts">
    import * as api from '#lib/api.ts';
    import type { Profile } from '#lib/types.ts';
    import Modal from '../Modal.svelte';

    let { profile = $bindable() }: { profile: Profile } = $props();

    let username = $state(profile.username);
    let designation = $state(profile.designation ?? '');
    let git = $state(profile.git ?? '');
    let website = $state(profile.website ?? '');
    let location = $state(profile.location ?? '');
    let saving = $state(false);

    let uploadOpen = $state(false);
    let uploadError = $state('');
    let fileInput = $state<HTMLInputElement>();

    async function save(e: SubmitEvent) {
        e.preventDefault();
        saving = true;
        const res = await api.post(`${profile.id}/update-profile`, { username, designation, git, website, location });
        const j = await api.json<{ success?: boolean; message?: string }>(res);
        saving = false;
        if (res.status === 200 && j.success) {
            profile = { ...profile, username, designation, git, website, location };
            alert('Profile data saved');
        } else {
            alert(j.message || 'Something went wrong, try again!');
        }
    }

    async function upload(e: SubmitEvent) {
        e.preventDefault();
        const file = fileInput?.files?.[0];
        if (!file) {
            uploadError = 'No image selected!';
            return;
        }
        if (file.size > 2 * 1024 * 1024) {
            uploadError = 'Max file size is 2MB';
            return;
        }
        const form = new FormData();
        form.append('file', file);
        const res = await api.upload(`${profile.id}/profile-image-upload`, form);
        const j = await api.json<{ url?: string; message?: string }>(res);
        if (res.status === 200 && j.url) {
            profile = { ...profile, image_url: j.url };
            uploadOpen = false;
        } else {
            uploadError = j.message || 'Upload failed, try again.';
        }
    }

    const fields = [
        { id: 'displayname', label: 'Display Name' },
        { id: 'designation', label: 'Designation' },
        { id: 'git', label: 'git' },
        { id: 'website', label: 'Website' },
        { id: 'location', label: 'Location' }
    ] as const;
</script>

<div class="mt-4 flex flex-col gap-6 md:flex-row">
    <div class="shrink-0">
        <p class="mb-2 font-bold">Profile Image (click to upload)</p>
        <button
            type="button"
            title="Change profile image"
            onclick={() => {
                uploadError = '';
                uploadOpen = true;
            }}
        >
            <img src={profile.image_url} alt="{profile.username}'s avatar" class="h-64 w-64 rounded object-cover" />
        </button>
    </div>
    <form onsubmit={save} class="flex-1 space-y-3">
        {#each fields as f (f.id)}
            <div class="grid items-center gap-1 sm:grid-cols-[140px_1fr]">
                <label for={f.id} class="text-sm font-bold">{f.label}</label>
                {#if f.id === 'displayname'}
                    <input id="displayname" name="displayname" class="input" bind:value={username} />
                {:else if f.id === 'designation'}
                    <input id="designation" name="designation" class="input" bind:value={designation} />
                {:else if f.id === 'git'}
                    <input id="git" name="git" class="input" bind:value={git} />
                {:else if f.id === 'website'}
                    <input id="website" name="website" class="input" bind:value={website} />
                {:else}
                    <input id="location" name="location" class="input" bind:value={location} />
                {/if}
            </div>
        {/each}
        <button type="submit" class="btn-primary" disabled={saving}>Submit Profile</button>
    </form>
</div>

<Modal bind:open={uploadOpen} title="Profile image">
    <form onsubmit={upload} class="space-y-3">
        {#if uploadError}<div class="alert-danger">{uploadError}</div>{/if}
        <input bind:this={fileInput} type="file" accept="image/*" id="image" class="block text-sm" />
        <div class="flex justify-end gap-2">
            <button type="button" class="btn-secondary" onclick={() => (uploadOpen = false)}>Close</button>
            <button type="submit" class="btn-primary">Upload</button>
        </div>
    </form>
</Modal>
