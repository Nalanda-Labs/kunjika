<script lang="ts">
    import { goto } from '$app/navigation';
    import { page } from '$app/state';
    import Icon from './Icon.svelte';

    let search = $state('');
    let menuOpen = $state(false);

    function submitSearch(e: SubmitEvent) {
        e.preventDefault();
        const q = search.trim();
        if (!q) return;
        menuOpen = false;
        goto(`/search?q=${encodeURIComponent(q)}`);
    }
</script>

<nav class="sticky top-0 z-30 border-b border-gray-200 bg-gray-50/95 backdrop-blur">
    <div class="mx-auto flex h-14 max-w-6xl items-center gap-4 px-4 md:pl-[248px]">
        <a class="text-xl font-normal text-gray-900" href="/">Kunjika</a>
        <button
            type="button"
            class="ml-auto inline-flex h-9 w-9 items-center justify-center rounded md:hidden"
            aria-label="Toggle navigation"
            aria-expanded={menuOpen}
            onclick={() => (menuOpen = !menuOpen)}
        >
            <Icon name={menuOpen ? 'x' : 'menu'} />
        </button>
        <div
            class="{menuOpen
                ? 'absolute inset-x-0 top-14 flex flex-col items-stretch gap-3 border-b border-gray-200 bg-gray-50 p-4'
                : 'hidden'} md:static md:flex md:flex-1 md:flex-row md:items-center md:gap-4 md:border-0 md:bg-transparent md:p-0"
        >
            <form class="md:flex-1" role="search" method="GET" action="/search" onsubmit={submitSearch}>
                <input
                    class="w-full rounded-full border border-gray-300 bg-white px-4 py-1.5 text-sm focus:border-brand focus:outline-none md:max-w-xl"
                    type="search"
                    name="q"
                    placeholder="Search questions"
                    aria-label="Search questions"
                    bind:value={search}
                />
            </form>
            <ul class="flex items-center gap-3">
                {#if page.data.user}
                    <li>
                        <a href="/users/{page.data.user.id}/{page.data.user.username}" aria-label="Your profile">
                            <img
                                src={page.data.user.image_url}
                                alt="profile"
                                class="h-8 w-8 rounded"
                            />
                        </a>
                    </li>
                    <li>
                        <form method="POST" action="/logout">
                            <button class="btn-primary py-1.5" type="submit">Sign Out</button>
                        </form>
                    </li>
                {:else}
                    <li><a href="/login" class="text-sm text-gray-700 hover:text-gray-900">Login</a></li>
                    <li><a href="/register" class="text-sm text-gray-700 hover:text-gray-900">Register</a></li>
                {/if}
            </ul>
        </div>
    </div>
</nav>
