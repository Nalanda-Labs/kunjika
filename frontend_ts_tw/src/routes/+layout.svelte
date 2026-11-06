<script lang="ts">
    import '../app.css';
    import { goto } from '$app/navigation';
    import { navigating, page } from '$app/state';
    import { onMount, type Snippet } from 'svelte';
    import * as api from '#lib/api.ts';
    import Footer from '#lib/components/Footer.svelte';
    import Nav from '#lib/components/Nav.svelte';
    import PreloadingIndicator from '#lib/components/PreloadingIndicator.svelte';
    import Sidebar from '#lib/components/Sidebar.svelte';

    let { children }: { children: Snippet } = $props();

    // Keep this below the access token's lifetime.
    const REFRESH_EVERY_MS = 55 * 60 * 1000;

    async function refreshSession() {
        if (!page.data.user) return;
        const res = await api.get('auth/refresh');
        if (res.status === 403) goto('/');
    }

    onMount(() => {
        // Refresh at once too: the browser may have been closed for a while.
        refreshSession();
        const timer = setInterval(refreshSession, REFRESH_EVERY_MS);
        return () => clearInterval(timer);
    });
</script>

{#if navigating.to}
    <PreloadingIndicator />
{/if}

<Nav />
<div class="flex min-h-[calc(100vh-56px)] flex-col md:flex-row md:items-start">
    <Sidebar />
    <div class="flex min-w-0 flex-1 flex-col">
        <main class="mx-auto w-full max-w-[900px] flex-1 px-4">
            {@render children()}
        </main>
        <Footer />
    </div>
</div>
