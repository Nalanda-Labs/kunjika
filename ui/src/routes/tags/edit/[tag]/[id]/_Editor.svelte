<script>
	import { goto } from "$app/navigation";
	import { session } from "$app/stores";
	import * as api from "$lib/api.js";
	import { onMount } from "svelte";
	import "bytemd/dist/index.min.css";

	export let tag;
	export let info;
	export let id;

	let inProgress = false;
	let Editor = null;
	let plugins = null;
	let value = "";

	async function onSubmit() {
		if ($session.user) {
			if (value.length < 20 || value.length > 100000) {
				M.toast({
					html: "Info should not be less than 20 or more than 100000 characters.",
				});
				return;
			}

			info = value;
			const response = await api.post(
				`tags/edit/${tag}/${id}`,
				{ tag_info: info },
				$session.user.xsrf_token
			);

			if (response.code !== 200) {
				M.toast({ html: response.msg });
			}
			await goto(`/tags/${tag}/${id}`);
		} else {
			M.toast({ html: "You are not logged in." });
		}
	}

	onMount(async () => {
		const bytemd = await import("bytemd");
		Editor = bytemd.Editor;
		let response = await api.get(
			`tags/edit/${tag}/${id}`,
			$session.user.xsrf_token
		);

		if (response.code === 200) {
			value = response.data;
		} else {
			M.toast({ html: "Error getting tag info!" });
		}
	});

	function handleChange(e) {
		value = e.detail.value;
	}
</script>

<div class="topic">
	<h3>About tag {tag}</h3>
	<hr />
	<form on:submit|preventDefault={onSubmit}>
		<svelte:component
			this={Editor}
			on:change={handleChange}
			mode="tab"
			{value}
		/>
		<div style="margin:30px" />
		<div class="b-wrapper">
			<!-- svelte-ignore a11y-invalid-attribute -->
			<button class="btn" type="submit">Edit</button>
		</div>
	</form>
</div>

<style>
	@media (max-width: 720px) {
		.topic {
			width: 100%;
		}
	}
	@media (max-width: 4096px) {
		.topic {
			width: 800px;
		}
	}
</style>
