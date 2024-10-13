<script context="module">
	export function preload({ params }, { user }) {
		console.log(params.id);
		console.log(user.id);
		if (params.id !== user.id) {
			goto('/questions');
		}
	}
</script>

<script>
	import { page } from '$app/stores';
	import * as api from '$lib/api.js';
	import { onMount } from 'svelte';

	async function getUser() {
		let xsrf_token = '';

		if (browser) {
			xsrf_token = getCookie('xsrf_token');
		}

		const response = await api.get(`user/${$page.params.id}`, xsrf_token);

		if (response.status === 200) {
			const text = await response.text();
			const j = text ? JSON.parse(text) : {};
			username = j.data.username;
			imageUrl = j.data.image_url;
			displayname = j.data.displayname;
			designation = j.data.designation;
			git = j.data.git;
			website = j.data.website;
			location = j.data.location;
			created_date = j.data.created_date;
		}
	}

	onMount(async () => await getUser());
</script>
