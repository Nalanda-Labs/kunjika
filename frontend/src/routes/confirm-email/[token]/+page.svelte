<script context="module">
	export async function load({ page }) {
		if (page.data.user) {
			return {
				status: 302,
				redirect: '/questions'
			};
		}

		return {};
	}
</script>

<script>
	import { page } from '$app/stores';
    import * as api from '$lib/api.js';
    import { onMount } from 'svelte';

    let message = '';
    let resendEmail = false;

    onMount(async () => {
        let response = await api.get(`confirm-email/${encodeURI($page.params.token)}`);

        if(response.status === 200) {
            response = await response.text();
			response = response ? JSON.parse(response) : {};

            if(response.success) {
                message = response.message;
            } else {
                message = response.message;
                resendEmail = true;
            }
        } else {
            message = `Something went wrong! Contact support with message "${response.message}"`;
        }
    });

    async function resendConfimationEmail() {
        let response = await api.get(`resend-confirmation-email/${encodeURI($page.params.token)}`);

        if(response.status === 200) {
            response = await response.text();
			response = response ? JSON.parse(response) : {};

            message = response.message;
        } else {
            message = `Something went wrong! Contact support with message "${response.message}"`;
        }
    }
</script>

<svelte:head>
	<title>Confirm Email ❤ Kunjika</title>
</svelte:head>

<div class="row justify-content-center align-items-center" style="margin-top:20px">
	<div class="col-12">
        <p>{message}</p>
        {#if resendEmail}
        <button class="btn btn-primary" on:click={resendConfimationEmail}>Resend Confirmation Email</button>
        {/if}
    </div>
</div>