<script lang="ts">
    import { onMount } from 'svelte';
    import * as api from '#lib/api.ts';
    import Icon from '#lib/components/Icon.svelte';
    import Pagination from '#lib/components/Pagination.svelte';
    import type { UserCard } from '#lib/types.ts';

    const PER_PAGE = 20;
    let users = $state<UserCard[]>([]);
    let count = $state(0);
    let page = $state(1);
    let searchQuery = $state('');
    let pages = $derived(Math.ceil(count / PER_PAGE));

    async function fetchUsers(after: string, back = false) {
        const res = await api.post('users', { last_user: after, ...(back ? { direction: 'back' } : {}) });
        if (res.status !== 200) return;
        const j = await api.json<{ data: { users: UserCard[] }; count: number }>(res);
        users = j.data.users;
        count = j.count;
    }

    const first = () => ((page = 1), fetchUsers(''));
    const next = () => users.length && ((page += 1), fetchUsers(users[users.length - 1].username));
    const prev = () =>
        users.length && ((page -= 1), page <= 1 ? first() : fetchUsers(users[0].username, true));

    async function search() {
        if (!searchQuery) return first();
        const res = await api.post('search-users', { username: searchQuery, users_per_page: PER_PAGE });
        if (res.status === 200) users = (await api.json<{ data: UserCard[] }>(res)).data;
    }

    onMount(first);
</script>

<svelte:head>
    <title>Users ❤ Kunjika</title>
</svelte:head>

<div class="mt-5">
    <h4 class="text-xl font-bold">Users</h4>
    <div class="relative mt-4 max-w-sm">
        <Icon name="search" size={16} class="absolute top-2.5 left-3 text-gray-400" />
        <input type="text" id="search" class="input pl-9" placeholder="Filter by user" bind:value={searchQuery} onkeyup={search} />
    </div>
    <div class="mt-5 grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-4">
        {#each users as u (u.id)}
            <div class="card flex items-start gap-3">
                <a href="/users/{u.id}/{u.username}" class="shrink-0">
                    <img src={u.image_url} alt="{u.displayname || u.username}'s avatar" class="h-12 w-12 rounded" />
                </a>
                <div class="min-w-0 text-xs">
                    <a href="/users/{u.id}/{u.username}" class="block truncate font-medium text-gray-900">{u.displayname || u.username}</a>
                    <p class="text-gray-600">{u.location}</p>
                    <p class="text-gray-600">{u.karma}</p>
                </div>
            </div>
        {/each}
    </div>
    {#if !searchQuery && pages > 1}
        <Pagination {page} {pages} onfirst={first} onprev={prev} onnext={next} />
    {/if}
</div>
