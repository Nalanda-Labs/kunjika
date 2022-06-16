<script context="module">
	export function preload({ params }) {
		let username = params.username;
		let id = params.id;
		return { username, id };
	}
</script>

<script>
	import { navOptions } from "./_Tabs.svelte";
	import { page } from "$app/stores";
	import { onMount } from "svelte";

	let username = $page.params.username;
	let id = $page.params.id;
	let tabName = "Profile";
	onMount(() => {
		var el = document.querySelector(".tabs");
		M.Tabs.init(el, {});
	});
</script>

<svelte:head>
	<title>All about {username}</title>
</svelte:head>

<div class="sm-topic" style="top:50px;position:relative">
	<div class="row">
		<div class="col s12">
			<ul class="tabs">
				{#each navOptions as page, i}
					{#if i == 0}
						<li class="tab col s3">
							<a class="active" href="#{page.page}">{page.page}</a
							>
						</li>
					{:else}
						<li class="tab col s3">
							<a href="#{page.page}">{page.page}</a>
						</li>
					{/if}
				{/each}
			</ul>
		</div>
		{#each navOptions as page}
			<div id={page.page} class="col s12">
				<svelte:component this={page.component} {username} {id} />
			</div>
		{/each}
	</div>
	<!-- <TabBar tabs={navOptions.map((option) => option.page)} let:tab>
		<Tab {tab} minWidth on:click={setTabName(tab)} style="color: #4285F4">
			<Label>{tab}</Label>
		</Tab>
	</TabBar> -->
</div>

<style>
	@media screen and (max-width: 480px) {
		.sm-topic {
			width: 100%;
		}
	}
	@media screen and (min-width: 481px) and (max-width: 4098px) {
		.md-topic {
			width: 70%;
		}
	}
</style>
