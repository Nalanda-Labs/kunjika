<script>
	import * as api from '$lib/api.js';
	import { onMount } from 'svelte';
	import markdownToTxt from 'markdown-to-txt';

	let tags = [];
	let count = 0;
	let pages = 0; // total no. of pages
	let page = 1; // current page
	let tags_per_page = 20;
	let tag = '';
	let searchQuery = '';

	onMount(async () => {
		let response = await api.post('tags', { tag: tag });

		if (response.status === 200) {
			response = JSON.parse(await response.text());
			tags = response.data.map((t) => t);
			response.data.map((t) => {
				t.info = markdownToTxt(t.info || '');
			});
			count = response.count;
			pages = Math.floor(count / tags_per_page);
			if (count % tags_per_page !== 0) {
				pages += 1;
			}
		}
	});

	async function nextPage() {
		page += 1;
		tag = tags[tags.length - 1].name;
		let response = await api.post('tags', { tag: tag });

		if (response.status === 200) {
			response = JSON.parse(await response.text());
			tags = response.data.map((t) => t);
			response.data.map((t) => {
				t.info = markdownToTxt(t.info || '');
			});
			count = response.count;
			pages = Math.floor(count / tags_per_page);
			if (count % tags_per_page !== 0) {
				pages += 1;
			}
		}
	}

	async function prevPage() {
		page -= 1;
		tag = tags[0].name;
		let response = await api.post('tags', { tag: tag, direction: 'back' });

		if (response.status === 200) {
			response = JSON.parse(await response.text());
			tags = response.data.map((t) => t);
			response.data.map((t) => {
				t.info = markdownToTxt(t.info || '');
			});
			count = response.count;
			pages = Math.floor(count / tags_per_page);
			if (count % tags_per_page !== 0) {
				pages += 1;
			}
		}
	}

	async function firstPage() {
		page = 1;
		tag = '';
		let response = await api.post('tags', { tag: tag });

		if (response.status === 200) {
			response = JSON.parse(await response.text());
			tags = response.data.map((t) => t);
			response.data.map((t) => {
				t.info = markdownToTxt(t.info || '');
			});
			count = response.count;
			pages = Math.floor(count / tags_per_page);
			if (count % tags_per_page !== 0) {
				pages += 1;
			}
		}
	}

	async function searchTags() {
		let pagination = document.getElementsByClassName('pagination');

		if (searchQuery && pagination[0]) {
			pagination[0].style.display = 'none';
		} else {
			pagination[0].style.display = 'flex';
			tags = [];
			pages = 0;
			await firstPage();
			return;
		}

		let response = await api.post('search-tags', { tag: searchQuery, tags_per_page });

		if (response.status === 200) {
			response = JSON.parse(await response.text());
			tags = response.data.map((t) => t);
			response.data.map((t) => {
				t.info = markdownToTxt(t.info || '');
			});
		}
	}
</script>

<svelte:head>
	<title>Tags ❤ Kunjika</title>
</svelte:head>

<div class="row justify-content-center align-items-center" style="margin-top:20px">
	<div class="col-12">
		<h4 class="text-xl font-bold">Tags</h4>
		<p>
			A tag is a keyword or label which is used to categorize similar questions. Tags make it easier
			to find similar questions.
		</p>
		<div class="input-group" style="max-width:50%">
			<span class="input-group-text"
				><i class="material-icons" style="display:inline">search</i></span
			>
			<input
				type="text"
				id="search"
				class="form-control"
				bind:value={searchQuery}
				on:keyup={() => searchTags()}
			/>
		</div>
		<div style="margin-top:20px" class="row">
			{#each tags as { name, info, post_count, weekly_count, daily_count }}
				<div class="col-sm-3" style="margin-bottom: 10px;">
					<div class="card">
						<div class="card-body">
							<span class="card-title"
								><a style="text-decoration:none" href="/questions/tagged/{encodeURIComponent(name)}"
									>{name}</a
								></span
							>
							{#if info}
								<p class="card-text" style="font-size: 12px;">{info.slice(0, 140)} ...</p>
							{:else}
								<p></p>
							{/if}
							<span style="font-size:11px"
								>{post_count || 0} questions, {daily_count || 0} today, {weekly_count || 0} this week</span
							>
						</div>
					</div>
				</div>
			{/each}
		</div>
		<div style="clear:both;margin:auto;width:100%;margin-top:20px" />
		<ul class="pagination" style="float:right">
			<!-- svelte-ignore a11y-invalid-attribute -->
			{#if page == 1}
				<li class="disabled"><i class="material-icons" title="first page">first_page</i></li>
				<li class="disabled">
					<i class="material-icons" title="previouse page">chevron_left</i>
				</li>
				{#if pages > 1}
					<li style="cursor:pointer" on:click={nextPage}>
						<i class="material-icons" title="next page">chevron_right</i>
					</li>
				{/if}
				{#if page != pages && pages != 0}
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
