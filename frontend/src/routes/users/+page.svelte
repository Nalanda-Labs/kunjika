<script>
	import * as api from '$lib/api.js';
	import { onMount, afterUpdate } from 'svelte';

	let users = [];
	let count = 0;
	let pages = 0; // total no. of pages
	let page = 1; // current page
	let users_per_page = 20;
	let user = '';

	async function getUsers() {
		let response = await api.post('users', { last_user: user });

		if (response.status === 200) {
			response = JSON.parse(await response.text());
			users = response.data.users.map((u) => u);
			count = response.count;
			pages = Math.floor(count / users_per_page);
			if (count % users_per_page !== 0) {
				pages += 1;
			}
		}
	}
	onMount(async () => await getUsers());

	async function nextPage() {
		page += 1;
		user = users[users.length - 1].username;
		let response = await api.post('users', { last_user: user });

		if (response.status === 200) {
			response = JSON.parse(await response.text());
			users = response.data.users.map((u) => u);
			count = response.count;
			pages = Math.floor(count / users_per_page);
			if (count % users_per_page !== 0) {
				pages += 1;
			}
		}
	}

	async function prevPage() {
		page -= 1;
		user = users[0].username;
		let response = await api.post('users', { last_user: user, direction: 'back' });

		if (response.status === 200) {
			response = JSON.parse(await response.text());
			users = response.data.users.map((u) => u);
			count = response.count;
			pages = Math.floor(count / users_per_page);
			if (count % users_per_page !== 0) {
				pages += 1;
			}
		}
	}

	async function firstPage() {
		page = 1;
		user = '';
		let response = await api.post('users', { last_user: user });

		if (response.status === 200) {
			response = JSON.parse(await response.text());
			users = response.data.users.map((u) => u);
			count = response.count;
			pages = Math.floor(count / users_per_page);
			if (count % users_per_page !== 0) {
				pages += 1;
			}
		}
	}
</script>

<svelte:head>
	<title>Users ❤ Kunjika</title>
</svelte:head>

<div class="row justify-content-center align-items-center" style="margin-top:20px">
	<div class="col-12">
		<h4 class="text-xl font-bold">Users</h4>
		<div class="input-group" style="max-width:50%">
			<span class="input-group-text" id="basic-addon1"
				><i class="material-icons" style="display:inline">search</i></span
			>
			<input type="text" class="form-control" placeholder="Filter by user" />
		</div>
		<div style="margin-top:20px" class="row">
			{#each users as { id, displayname, username, location, karma, image_url }}
				<div class="col-sm-3">
					<div class="card">
						<div class="card-body">
							<a href="/users/{id}/{username}" style="display:flex;float:left">
								<img
									src={image_url}
									alt="{displayname}'s avatar"
									style="display:flex;float:left;height:48px;width:48px"
								/>
							</a>
							<p
								style="display:flex;flex-wrap: wrap !important;padding-left:5px;margin-bottom:5px;font-size:12px"
							>
								{displayname || username}
							</p>
							<p
								style="display:flex;color:#666;padding-left:5px;margin-top:-5px;margin-bottom:5px;font-size:12px"
							>
								{location}
							</p>
							<p
								style="display:flex;color:#666;padding-left:5px;margin-top:-5px;margin-bottom:5px;font-size:12px"
							>
								{karma}
							</p>
						</div>
					</div>
				</div>
			{/each}
		</div>
		<div style="clear:both;margin:auto;width:100%;margin-top:20px" />
		<ul class="pagination" style="float:right">
			{#if page == 1}
				<li class="disabled"><i class="material-icons" title="first page">first_page</i></li>
				<li class="disabled">
					<i class="material-icons" title="previouse page">chevron_left</i>
				</li>
				{#if page != pages}
					<li style="cursor:pointer" on:click={nextPage}>
						<i class="material-icons" title="next page">chevron_right</i>
					</li>
				{/if}
				{#if page == pages}
					<li class="disabled"><i class="material-icons" title="last page">last_page</i></li>
				{/if}
			{:else if page != pages}
				<li style="cursor:pointer" on:click={firstPage}>
					<i class="material-icons" title="first page">first_page</i>
				</li>
				<li style="cursor:pointer" on:click={prevPage}>
					<i class="material-icons" title="previouse page">chevron_left</i>
				</li>
				<li style="cursor:pointer" on:click={nextPage}>
					<i class="material-icons" title="next page">chevron_right</i>
				</li>
			{:else if page == pages}
				<li style="cursor:pointer" on:click={firstPage}>
					<i class="material-icons" title="first page">first_page</i>
				</li>
				<li style="cursor:pointer" on:click={prevPage}>
					<i class="material-icons" title="previouse page">chevron_left</i>
				</li>
				<li class="disabled"><i class="material-icons" title="next page">chevron_right</i></li>
				<li class="disabled"><i class="material-icons" title="last page">last_page</i></li>
			{/if}
		</ul>
	</div>
</div>
