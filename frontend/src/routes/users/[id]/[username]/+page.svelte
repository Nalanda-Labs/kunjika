<script context="module">
	export function preload({ params }, { user }) {}
</script>

<script>
	import { page } from '$app/stores';
	import * as api from '$lib/api.js';
	import { onMount } from 'svelte';
	import Summary from './_Summary.svelte';

	let edit;
	let questions;
	let answers;
	let bookmarks;
	let deleteSelf;

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

	async function loadEdit() {
		edit = await import('./_Edit.svelte');
	}

	async function loadQuestions() {
		questions = await import('./_Questions.svelte');
	}

	async function loadAnswers() {
		answers = await import('./_Answers.svelte');
	}

	async function loadBookmarks() {
		bookmarks = await import('./_Bookmarks.svelte');
	}

	async function loadDeleteSelf() {
		deleteSelf = await import('./_DeleteSelf.svelte');
	}

	onMount(async () => await getUser());
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
					<i class="fa-brands fa-github" style="margin-right:5px"></i>{git.split('//').slice(-1)}
				</a>
			{:else if $page.data.user != null && id == $page.data.user.id}
				<div contenteditable="true" bind:textContent={git}>git url</div>
			{/if}
			{#if website}
				<a href={website}>
					&nbsp;<i class="fa-solid fa-link"></i>
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
		<div style="clear:both;"></div>
		<div style="margin-top:20px">
			<ul class="nav nav-tabs">
				<li class="nav-item">
					<a class="nav-link active" href="#activity" data-bs-toggle="tab">Activity</a>
				</li>
				{#if $page.data.user != null && id == $page.data.user.id}
					<li class="nav-item">
						<a class="nav-link" href="#bookmarks" data-bs-toggle="tab" on:click={loadBookmarks}
							>Bookmarks</a
						>
					</li>
					<li class="nav-item">
						<a class="nav-link" href="#settings" data-bs-toggle="tab" on:click={loadEdit}
							>Settings</a
						>
					</li>
				{/if}
			</ul>
			<div style="margin-top:10px"></div>
			<div class="tab-content">
				<!-- Repo -->
				<div class="tab-pane active" id="activity">
					<ul class="nav nav-tabs" id="repoTabs">
						<li><a class="nav-link active" href="#summary1" data-bs-toggle="tab">Summary</a></li>
						<li>
							<a class="nav-link" href="#questions" data-bs-toggle="tab" on:click={loadQuestions}
								>Questions</a
							>
						</li>
						<li>
							<a class="nav-link" href="#answers" data-bs-toggle="tab" on:click={loadAnswers}
								>Answers</a
							>
						</li>
					</ul>

					<!-- Repo Tabs -->

					<div class="tab-content">
						<div class="tab-pane active" id="summary1">
							<Summary {id} {username} />
						</div>
						<div class="tab-pane" id="questions">
							{#if questions}
								{#await questions then { default: Questions }}
									<Questions {id} />
								{/await}
							{/if}
						</div>
						<div class="tab-pane" id="answers">
							{#if answers}
								{#await answers then { default: Answers }}
									<Answers {id} />
								{/await}
							{/if}
						</div>
					</div>
				</div>
				<div class="tab-pane" id="bookmarks">
					{#if bookmarks}
						{#await bookmarks then { default: Bookmarks }}
							<Bookmarks {id} />
						{/await}
					{/if}
				</div>
				<div class="tab-pane" id="settings">
					<ul class="nav nav-tabs">
						<li>
							<a class="nav-link active" href="#edit-profile" data-bs-toggle="tab"> Edit </a>
						</li>
						<li><a class="nav-link" href="#delete-profile" data-bs-toggle="tab"  on:click={loadDeleteSelf}>Delete</a></li>
						<li>
							<a class="nav-link" href="#settings1" data-bs-toggle="tab"
								>Settings</a
							>
						</li>
					</ul>
					<div class="tab-content">
						<div class="tab-pane active" id="edit-profile">
							{#if edit}
								{#await edit then { default: Edit }}
									<Edit {id} {username} {imageUrl} {location} {git} {website} {designation} />
								{/await}
							{/if}
						</div>
						<div class="tab-pane" id="delete-profile">
							{#if deleteSelf}
								{#await deleteSelf then { default: DeleteSelf }}
									<DeleteSelf {id} {username} />
								{/await}
							{/if}
						</div>
						<div class="tab-pane" id="settings1">Not implemented</div>
					</div>
				</div>
			</div>
		</div>
	</div>
</div>
