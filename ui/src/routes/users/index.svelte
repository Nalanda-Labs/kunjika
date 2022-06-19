<script context="module">
	export function preload({ params }) {}
</script>

<script>
	import { onMount } from "svelte";
	import * as api from "$lib/api.js";
	import InfiniteLoading from "svelte-infinite-loading";

	let users = [];
	let data = [];

	async function infiniteHandler({ detail: { loaded, complete } }) {
		let last_user = "";
		if (users.length) {
			last_user = users[users.length - 1].username;
		}
		let response = await api.post(`users`, { last_user });
		if (response.code == 200 && response.data.users) {
			users = response.data.users;
		}
		if (users.length) {
			data = [...data, ...users];
			loaded();
		} else {
			complete();
		}
	}
</script>

<svelte:head>
	<title>Users ❤ Kunjika</title>
</svelte:head>

<div class="users">
	<h3>Users</h3>
	<div class="row">
		{#each data as { id, username, name, location, image_url }}
			<div class="col-md-3 col-sm-6">
				<img
					src="{image_url}?s=120"
					alt="{username}'s proile image"
					width="120px;"
				  />
				<br />
				<a
					class="anchor"
					style="font-size: 12px;margin-left:5px"
					href="/user/{id}/{username}">{name || username}
        </a>
				<p style="font-size: 12px;margin-left:5px;margin-top:0px">
					{location}
				</p>
			</div>
		{/each}
		<InfiniteLoading on:infinite={infiniteHandler} />
	</div>
</div>

<style>
	@media (max-width: 720px) {
		.users {
			width: 100%;
		}
	}
	@media (max-width: 4096px) {
		.users {
			width: 800px;
		}
	}
</style>
