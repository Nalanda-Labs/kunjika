<script>
    import { goto } from '$app/navigation';
    import { onDestroy, onMount } from 'svelte';
    import { page, navigating } from '$app/stores';
    import Nav from './Nav.svelte';
    import Sidebar from './Sidebar.svelte';
    import PreloadingIndicator from './PreloadingIndicator.svelte';
    import * as api from '../lib/api.js';
    import { browser } from '$app/environment';
    import '../app.css';
    import Footer from './Footer.svelte';

    const refresh = setInterval(
        async () => {
            if (browser && $page.data.user) {
                const resp = await api.get('auth/refresh', {});

                if (resp.status === 403) {
                    goto('/');
                }

                // TODO: remove this hardcoding. keep this interval less than access token max age
            }
        },
        55 * 60 * 1000
    );

    onMount(async () => {
        // this immediate refresh is for the reason when user will close the
        // browser and reopen it which can lead to three cases for both
        // the access token and refresh token
        if ($page.data.user) {
            const resp = await api.get('auth/refresh', {});

            if (resp.status === 403) {
                goto('/');
            }
        }
        return async () => await refresh();
    });

    onDestroy(async () => {
        clearInterval(refresh);
    });
</script>

{#if $navigating}
    <PreloadingIndicator />
{/if}

<Nav />
<div class="app-shell">
    <Sidebar />
    <div class="content-column">
        <div class="container">
            <main>
                <slot />
            </main>
        </div>
        <Footer />
    </div>
</div>

<style>
    .app-shell {
        display: flex;
        align-items: flex-start;
        min-height: calc(100vh - 56px);
    }

    .content-column {
        flex: 1 1 auto;
        min-width: 0;
        display: flex;
        flex-direction: column;
    }

    .container {
        max-width: 900px !important;
    }

    @media (max-width: 767.98px) {
        .app-shell {
            flex-direction: column;
        }
    }
</style>
