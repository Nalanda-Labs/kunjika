<script lang="ts">
    import { goto } from '$app/navigation';
    import { afterNavigate } from '$app/navigation';
    import { page } from '$app/state';
    import { refreshUnread, unread } from '#lib/notifications.svelte.ts';
    import Icon from './Icon.svelte';

    // Keep the bell current: checked on load, after every navigation, and
    // then every 30 seconds while the tab is visible, so a new answer shows
    // up without the reader having to click anything. (Polling rather than a
    // pushed stream: the API authenticates with a header that a browser
    // EventSource cannot send.)
    const POLL_MS = 30_000;
    afterNavigate(() => {
        if (page.data.user) refreshUnread();
    });
    $effect(() => {
        if (!page.data.user) return;
        const check = () => {
            if (document.visibilityState === 'visible') refreshUnread();
        };
        const timer = setInterval(check, POLL_MS);
        // Catch up straight away when the reader comes back to the tab.
        document.addEventListener('visibilitychange', check);
        return () => {
            clearInterval(timer);
            document.removeEventListener('visibilitychange', check);
        };
    });

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
                        <a
                            href="/notifications"
                            class="relative inline-flex h-8 w-8 items-center justify-center rounded text-gray-700 hover:bg-gray-100"
                            aria-label={unread.count > 0 ? `Notifications (${unread.count} unread)` : 'Notifications'}
                            title="Notifications"
                        >
                            <Icon name="bell" size={20} />
                            {#if unread.count > 0}
                                <span
                                    class="absolute -top-1 -right-1 min-w-4 rounded-full bg-danger px-1 text-center text-[10px] leading-4 text-white"
                                    id="unreadBadge">{unread.count > 99 ? '99+' : unread.count}</span
                                >
                            {/if}
                        </a>
                    </li>
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
