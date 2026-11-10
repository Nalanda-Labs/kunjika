<script lang="ts">
    import { page } from '$app/state';
    import { onMount } from 'svelte';
    import * as api from '#lib/api.ts';
    import Icon from '#lib/components/Icon.svelte';
    import PagedQuestions from '#lib/components/PagedQuestions.svelte';
    import DeleteProfile from '#lib/components/profile/DeleteProfile.svelte';
    import EditProfile from '#lib/components/profile/EditProfile.svelte';
    import Summary from '#lib/components/profile/Summary.svelte';
    import UserPosts from '#lib/components/profile/UserPosts.svelte';
    import { isReviewer, type Profile, type QuestionRow } from '#lib/types.ts';

    type Tab = 'activity' | 'bookmarks' | 'settings';
    type Sub = 'summary1' | 'questions' | 'answers';
    type Setting = 'edit-profile' | 'delete-profile';

    let id = $derived(page.params.id ?? '');
    let isSelf = $derived(!!page.data.user && String(page.data.user.id) === id);

    let profile = $state<Profile | null>(null);
    let tab = $state<Tab>('activity');
    let sub = $state<Sub>('summary1');
    let setting = $state<Setting>('edit-profile');
    // Panels mount the first time they are opened, then stay.
    let opened = $state(new Set<string>(['summary1', 'edit-profile']));

    function open(e: MouseEvent, set: () => void, key: string) {
        e.preventDefault();
        set();
        opened.add(key);
        opened = new Set(opened);
    }

    onMount(async () => {
        const res = await api.get(`user/${id}/${page.params.username}`);
        if (res.status === 200) profile = (await api.json<{ data: Profile }>(res)).data;
    });

    const loadQuestions = async (uat: string, back: boolean) => {
        const res = await api.post(`${id}/get-questions-by-user/`, { uat, ...(back ? { direction: 'back' } : {}) });
        const j = await api.json<{ data?: QuestionRow[]; count?: number; message?: string }>(res);
        if (res.status === 200) return { questions: j.data ?? [], count: j.count ?? 0 };
        if (j.message === api.NO_ROWS) return { questions: [], count: 0 };
        throw new Error(j.message || 'Could not load questions.');
    };

    let suspended = $derived(profile?.status === 'suspended');

    /** Moderators: suspend this account, or reinstate it. */
    async function setSuspended(next: boolean) {
        if (!profile) return;
        const question = next
            ? `Suspend ${profile.username}? They will be signed out and unable to sign in or post.`
            : `Reinstate ${profile.username}?`;
        if (!confirm(question)) return;
        const res = await api.post(`users/${profile.id}/suspend/`, { suspended: next });
        const j = await api.json<{ message?: string }>(res);
        if (res.status === 200) profile.status = next ? 'suspended' : 'normal';
        else alert(j.message || 'Could not update the account.');
    }

    function link(href: string | null | undefined): string {
        if (!href) return '';
        return /^https?:\/\//.test(href) ? href : `https://${href}`;
    }
</script>

<svelte:head>
    <title>Profile ❤ Kunjika</title>
</svelte:head>

{#if profile}
    <div class="mt-5">
        <div class="flex gap-4">
            <img src={profile.image_url} alt="{profile.username}'s avatar" class="h-32 w-32 rounded object-cover" />
            <div class="min-w-0">
                <h4 class="text-2xl font-normal break-words">
                    {profile.username}
                    {#if suspended}
                        <span class="ml-2 rounded bg-red-100 px-2 py-0.5 align-middle text-xs font-medium text-red-800" id="suspendedBadge"
                            >suspended</span
                        >
                    {/if}
                </h4>
                <p class="text-gray-500">Member since {new Date(profile.cat * 1000).toISOString().slice(0, 10)}</p>
                {#if profile.designation}<p class="text-sm text-gray-700">{profile.designation}</p>{/if}
                <p class="mt-2 flex flex-wrap gap-4 text-sm">
                    {#if profile.git}<a class="link inline-flex items-center gap-1" href={link(profile.git)} target="_blank" rel="noopener noreferrer"><Icon name="github" size={14} />git</a>{/if}
                    {#if profile.website}<a class="link" href={link(profile.website)} target="_blank" rel="noopener noreferrer">website</a>{/if}
                    {#if profile.location}<span class="text-gray-600">{profile.location}</span>{/if}
                </p>
                {#if isReviewer(page.data.user) && !isSelf}
                    <button
                        type="button"
                        class="{suspended ? 'btn-secondary' : 'btn-danger'} mt-3 text-sm"
                        onclick={() => setSuspended(!suspended)}>{suspended ? 'Reinstate user' : 'Suspend user'}</button
                    >
                {/if}
            </div>
        </div>

        <div class="mt-5 flex border-b border-gray-200">
            <a href="#activity" class="tab-link" class:active={tab === 'activity'} onclick={(e) => open(e, () => (tab = 'activity'), 'activity')}>Activity</a>
            {#if isSelf}
                <a href="#bookmarks" class="tab-link" class:active={tab === 'bookmarks'} onclick={(e) => open(e, () => (tab = 'bookmarks'), 'bookmarks')}>Bookmarks</a>
                <a href="#settings" class="tab-link" class:active={tab === 'settings'} onclick={(e) => open(e, () => (tab = 'settings'), 'settings')}>Settings</a>
            {/if}
        </div>

        <div id="activity" hidden={tab !== 'activity'}>
            <div class="mt-3 flex border-b border-gray-200">
                <a href="#summary1" class="tab-link" class:active={sub === 'summary1'} onclick={(e) => open(e, () => (sub = 'summary1'), 'summary1')}>Summary</a>
                <a href="#questions" class="tab-link" class:active={sub === 'questions'} onclick={(e) => open(e, () => (sub = 'questions'), 'questions')}>Questions</a>
                <a href="#answers" class="tab-link" class:active={sub === 'answers'} onclick={(e) => open(e, () => (sub = 'answers'), 'answers')}>Answers</a>
            </div>
            <div id="summary1" hidden={sub !== 'summary1'}><Summary {id} /></div>
            <div id="questions" hidden={sub !== 'questions'}>
                {#if opened.has('questions')}<PagedQuestions load={loadQuestions} empty="No questions asked yet." />{/if}
            </div>
            <div id="answers" hidden={sub !== 'answers'}>
                {#if opened.has('answers')}<UserPosts path="{id}/get-answers-by-user/" empty="No answers yet." />{/if}
            </div>
        </div>

        {#if isSelf}
            <div id="bookmarks" hidden={tab !== 'bookmarks'}>
                {#if opened.has('bookmarks')}
                    <UserPosts path="{id}/get-bookmarks-by-user/" extra={{ bookmarks_per_page: 30 }} empty="No bookmarks yet." />
                {/if}
            </div>
            <div id="settings" hidden={tab !== 'settings'}>
                <div class="mt-3 flex border-b border-gray-200">
                    <a href="#edit-profile" class="tab-link" class:active={setting === 'edit-profile'} onclick={(e) => open(e, () => (setting = 'edit-profile'), 'edit-profile')}>Edit</a>
                    <a href="#delete-profile" class="tab-link" class:active={setting === 'delete-profile'} onclick={(e) => open(e, () => (setting = 'delete-profile'), 'delete-profile')}>Delete</a>
                </div>
                <div id="edit-profile" hidden={setting !== 'edit-profile'}>
                    {#if opened.has('settings')}<EditProfile bind:profile={profile} />{/if}
                </div>
                <div hidden={setting !== 'delete-profile'}>
                    {#if opened.has('delete-profile')}<DeleteProfile id={profile.id} />{/if}
                </div>
            </div>
        {/if}
    </div>
{/if}
