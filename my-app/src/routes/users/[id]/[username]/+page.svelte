<script context="module">
	export function preload({ params }, { user }) {}
</script>

<script>
	import { page } from '$app/stores';
	import * as api from '$lib/api.js';
	import { onMount } from 'svelte';

	let joining_date = $page.data.user.created_date.split('T')[0];

	let id = $page.params.id;
	let username = $page.params.username;
	let imageUrl = '';
	let displayname = '';
	let designation = '';
	let git = '';
	let website = '';
	let location = '';
	let created_date = '';

	onMount(async () => {
		const response = await api.get(`user/${$page.params.id}/${$page.params.username}`);

		if (response.status === 200) {
			const text = await response.text();
			const j = text ? JSON.parse(text) : {};
			imageUrl = j.data.image_url;
			displayname = j.data.displayname;
			designation = j.data.designation;
			git = j.data.git;
			website = j.data.website;
			location = j.data.location;
			created_date = j.data.created_date;
		}
	});
</script>

<svelte:head>
	<title>Profile ❤ Kunjika</title>
</svelte:head>

<div class="row">
	<div class="col s12 m8 offset-m4 xl10 offset-xl2">
		<div class="row">
			<a
				href="/users/{$page.data.user.id}/{$page.data.user.username}"
				style="display:flex;float:left"
			>
				<img
					src="{imageUrl}?s=128"
					alt="{displayname}'s avatar"
					style="display:flex;float:left"
				/>
			</a>
			<h4 style="display:flex;flex-wrap: wrap !important;padding-left:5px">
				{displayname || username}
			</h4>
			<h5 style="display:flex;color:#666;padding-left:5px;margin-top:-5px">
				{designation}
			</h5>
			<p style="display:flex;color:#666;padding-left:5px;margin-top:-5px">
				Member since {created_date.split('T')[0]}
			</p>
			<p style="display:flex;color:#666;padding-left:5px;margin-top:-5px;float:left">
				{#if git}
					<a href={git} title={git.split('/').slice(-1)}>
						<i
							class="material-icons"
							style="display:flex!important;font-size:15px;margin-right:5px;margin-left:-45px"
							>github</i
						></a
					>
				{/if}
				{#if $page.data.user.website}
					<a href={$page.data.user.website}
						><i class="material-icons" style="display:inline!important;font-size:15px;">link</i>
						{$page.data.user.website.split('//').slice(-1)}
					</a>
				{/if}
				{#if $page.data.user.location}
					<a href={$page.data.user.lcoation}
						><i class="material-icons" style="display:inline!important;font-size:15px"
							>location_on</i
						>
						{$page.data.user.location}
					</a>
				{/if}
			</p>
			<p style="display:flex;color:#666;padding-left:5px;float:right">
				{#if $page.params.id == $page.data.user.id}
					<a class="btn" href="/users/edit/{$page.params.id}">Edit Profile</a>
				{/if}
			</p>
		</div>
	</div>
</div>
