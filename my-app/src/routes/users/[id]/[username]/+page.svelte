<script context="module">
	export function preload({ params }, { user }) {}
</script>

<script>
	import { page } from '$app/stores';
	import * as api from '$lib/api.js';
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
					src="{$page.data.user.image_url}?s=128"
					alt="{$page.data.user.displayname}'s avatar"
					style="display:flex;float:left"
				/>
			</a>
			<h4
				style="display:flex;flex-wrap: wrap !important;padding-left:5px"
			>
				{$page.data.user.displayname || $page.data.user.username}
			</h4>
			<h5 style="display:flex;color:#666;padding-left:5px;margin-top:-10px">Architect</h5>
		</div>
	</div>
</div>
