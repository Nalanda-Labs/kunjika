<script context="module">
	export function preload({ params }, { user }) {}
</script>

<script>
	import { page } from '$app/stores';
	import * as api from '$lib/api.js';
	import { afterUpdate, onMount } from 'svelte';
	import Edit from './_Edit.svelte';

	let id = $page.params.id;
	$: username = '';
	$: imageUrl = '';
	$: designation = '';
	$: git = 'git url';
	$: website = '';
	$: location = '';
	let created_date = '';
	let cat = '';

	async function getUser() {
		const response = await api.get(`user/${$page.params.id}/${$page.params.username}`);

		if (response.status === 200) {
			const text = await response.text();
			const j = text ? JSON.parse(text) : {};
			username = j.data.username;
			imageUrl = j.data.image_url;
			designation = j.data.designation;
			git = j.data.git;
			website = j.data.website;
			location = j.data.location;
			created_date = j.data.created_date;
			cat = new Date(j.data.cat * 1000).toISOString();
		}
	}

	onMount(async () => await getUser());
	afterUpdate(async () => await getUser());
</script>

<svelte:head>
	<title>Profile ❤ Kunjika</title>
</svelte:head>

<div class="row justify-content-center align-items-center" style="margin-top:20px">
	<div class="col-12">
		<a href="/users/{id}/{username}" style="display:flex;float:left">
			<img
				src={imageUrl}
				alt="{username}'s avatar"
				style="display:flex;float:left;width:128px;height:128px"
			/>
		</a>
		<h4 style="display:flex;flex-wrap: wrap !important;padding-left:5px">
			{username}
		</h4>
		<h5 style="display:flex;color:#666;padding-left:5px;margin-top:-5px">
			{designation}
		</h5>
		<p style="display:flex;color:#666;padding-left:5px;margin-top:-5px">
			Member since {cat.split('T')[0]}
		</p>
		<p style="display:flex;color:#666;padding-left:5px;margin-top:-5px;float:left">
			{#if git}
				<a href={git} title={git.split('//').slice(-1)}>
					<i class="fa-brands fa-github" style="margin-right:5px"></i>{git.split('//').slice(-1)}</a
				>
			{:else if $page.data.user != null && id == $page.data.user.id}
				<div contenteditable="true" bind:textContent={git}>git url</div>
			{/if}
			{#if website}
				<a href={website}>
					<i class="fa-solid fa-link"></i>
					{website.split('//').slice(-1)}
				</a>
			{/if}
			{#if location}
				<a href={location}>
					<i class="fa-solid fa-location-dot" style="margin-left:5px"></i>
					{location}
				</a>
			{/if}
		</p>
		<div style="clear:both;" />
		<div style="margin-top:20px">
			<ul class="nav nav-tabs">
				<li class="nav-item">
					<a class="nav-link" aria-current="page" href="#profile" data-bs-toggle="tab">Profile</a>
				</li>
				<li class="nav-item">
					<a class="nav-link active" href="#activity" data-bs-toggle="tab">Activity</a>
				</li>
				<li class="nav-item">
					<a class="nav-link" href="#bookmarks" data-bs-toggle="tab">Bookmarks</a>
				</li>
				{#if $page.data.user != null && id == $page.data.user.id}
					<li class="nav-item">
						<a class="nav-link" href="#settings" data-bs-toggle="tab">Settings</a>
					</li>
				{/if}
			</ul>
			<div style="margin-top:10px" />
			<div class="tab-content">
				<!-- Repo -->
				<div class="tab-pane active" id="activity">
					<!-- Repo Tabs --->
					<ul class="nav nav-tabs" id="repoTabs">
						<li><a class="nav-link active" href="#summary1" data-bs-toggle="tab">Summary</a></li>
						<li><a class="nav-link" href="#questions" data-bs-toggle="tab">Questions</a></li>
						<li><a class="nav-link" href="#answers" data-bs-toggle="tab">Answers</a></li>
					</ul>

					<!-- Repo Tabs -->

					<div class="tab-content">
						<div class="tab-pane active" id="summary1">Not implemented</div>
						<div class="tab-pane" id="questions">Not implemented</div>
						<div class="tab-pane" id="answers">Not implemented</div>
					</div>
				</div>
				<div class="tab-pane" id="profile">Not implemented</div>
				<div class="tab-pane" id="bookmarks">Not implemented</div>
				<div class="tab-pane" id="settings">
					<ul class="nav nav-tabs">
						<li>
							<a class="nav-link active" href="#edit-profile" data-bs-toggle="tab"> Edit </a>
						</li>
						<li><a class="nav-link" href="#delete-profile" data-bs-toggle="tab">Delete</a></li>
						<li><a class="nav-link" href="#settings1" data-bs-toggle="tab">Settings</a></li>
					</ul>
					<div class="tab-content">
						<div class="tab-pane active" id="edit-profile">
							<Edit {id} {username} {imageUrl} {location} {git} {website} {designation} />
						</div>
						<div class="tab-pane" id="delete-profile">Not implemented</div>
						<div class="tab-pane" id="settings1">Not implemented</div>
					</div>
				</div>
			</div>
		</div>
	</div>
</div>
