<script>
	import { onMount } from "svelte";
	import * as api from "$lib/api.js";
	import InfiniteLoading from "svelte-infinite-loading";

	let tags = [];
	let data = [];

	async function infiniteHandler({ detail: { loaded, complete } }) {
		let last_tag = "";
		if (tags.length) {
			last_tag = tags[tags.length - 1].name;
		}
		let response = await api.post(`tags`, {
			tag: last_tag,
		});
		if (response.code === 200 && response.data) {
			tags = response.data;
		}
		if (tags.length) {
			data = [...data, ...tags];
			loaded();
		} else {
			complete();
		}
	}
</script>

<svelte:head>
	<title>Tags • Arth</title>
</svelte:head>

<div class="tags">
	<h3>Tags</h3>
	<p>
		A tag lets you know technologies, concepts involved with a
		topic/question. It helps you find similar questions or watch a
		particular tag for topics/questions.
	</p>
	<div class="row">
		{#each data as { id, info, name, post_count }}
			<div
				class="card"
				style="min-width:250px;max-width:250px;float:left;margin-right:10px"
			>
				<div class="card-content">
					<a class="card-title" style="display:inline" href="/topics/tagged/{name}">{name}</a
					>
					<p>
						{#if info}
							{info.slice(0, 50)}...
						{/if}
					</p>
					<p style="color: #777;">{post_count} questions <a style="float:right" href="/tag/edit/{id}">Edit</p>
				</div>
			</div>
		{/each}
		<InfiniteLoading on:infinite={infiniteHandler} />
	</div>
</div>

<style>
	@media (max-width: 720px) {
		.tags {
			width: 100%;
		}
	}
	@media (max-width: 4096px) {
		.tags {
			width: 800px;
		}
	}
</style>
