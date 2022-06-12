<script>
    import { onMount } from "svelte";
    import * as api from "$lib/api.js";

    let response = {};
    let verified = false;

    onMount(async () => {
        // remove leading slash else it will have two slashes
        response = await api.get(window.location.pathname.slice(1));

        if (response.detail) {
            verified = false;
        } else if (response.success) {
            verified = true;
        }
    });
</script>

<svelte:head>
    <title>Confirm Email ❤ Arth</title>
</svelte:head>

<main>
    {#if verified}
        <p>
            Your email has been verified! Proceed to <a href="/login">Login</a>
        </p>
    {:else}
        <p>Your email could not be verified! {response.details}</p>
    {/if}
</main>
