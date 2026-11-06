<script lang="ts">
    import { page } from '$app/state';
    import { onMount } from 'svelte';
    import * as api from '#lib/api.ts';

    let message = $state('');
    let canResend = $state(false);
    let token = $derived(encodeURIComponent(page.params.token ?? ''));

    onMount(async () => {
        const res = await api.get(`confirm-email/${token}`);
        const j = await api.json<{ success?: boolean; message?: string }>(res);
        if (res.status === 200) {
            message = j.message ?? '';
            canResend = !j.success;
        } else {
            message = `Something went wrong! Contact support with message "${j.message ?? res.statusText}"`;
            // An expired link can be replaced; the backend's message says so.
            canResend = /expired/i.test(j.message ?? '');
        }
    });

    async function resend() {
        const res = await api.get(`resend-confirmation-email/${token}`);
        const j = await api.json<{ message?: string }>(res);
        message =
            res.status === 200
                ? (j.message ?? '')
                : `Something went wrong! Contact support with message "${j.message ?? res.statusText}"`;
        canResend = false;
    }
</script>

<svelte:head>
    <title>Confirm Email ❤ Kunjika</title>
</svelte:head>

<div class="mt-5 space-y-4">
    <p>{message}</p>
    {#if canResend}
        <button class="btn-primary" onclick={resend}>Resend Confirmation Email</button>
    {/if}
</div>
