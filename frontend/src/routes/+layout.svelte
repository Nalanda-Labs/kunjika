<script>
	import { goto } from '$app/navigation';
	import { onDestroy, onMount } from 'svelte';
	import { page, navigating } from '$app/stores';
	import Nav from './Nav.svelte';
	import PreloadingIndicator from './PreloadingIndicator.svelte';
	import * as api from '../lib/api.js';
	import { browser } from '$app/environment';
	import '../app.css';

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
		1 * 60 * 1000
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
		return async () => await refresh;
	});

	onDestroy(async () => {
		clearInterval(refresh);
	});
</script>

{#if $navigating}
	<PreloadingIndicator />
{/if}

<Nav />
<div class="container">
	<main>
		<slot />
	</main>
</div>
