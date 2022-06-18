<script>
	import { goto } from "$app/navigation";
	import { session } from "$app/stores";
	import * as api from "$lib/api.js";
	import { onMount } from "svelte";
	import "bytemd/dist/index.min.css";

	export let topic;
	export let id;

	let inProgress = false;
	let errors;
	let Editor = null;
	let plugins = null;
	let value = "";
	let Tags = null;
	let count = 0;

	async function onSubmit() {
		if ($session.user) {
			inProgress = true;
			if (topic.title != "") {
				if (topic.title < 6 || topic.title > 256) {
					M.toast({
						html: "Title should not be less than 6 or more than 256 characters.",
					});
					return;
				}
			}
			if (value.length < 20 || value.length > 100000) {
				M.toast({
					html: "Topic should not be less than 20 or more than 100000 characters.",
				});
				return;
			}

			topic.body = value;
			if (topic.title && topic.taglist.length == 0) {
				M.toast({ html: "At least one tag should be supplied." });
			}

			const response = await api.post(
				`edit-post/${id}`,
				{ topic },
				$session.user.xsrf_token
			);

			if (response.code !== 200) {
				M.toast({ html: response.msg });
			}
			if (response.id && response.slug) {
				id = response.id;
				await goto(`/questions/${id}`);
			}

			inProgress = false;
		} else {
			M.toast({ html: "You are not logged in." });
		}
	}

	onMount(async () => {
		if ($session.user) {
			const bytemd = await import("bytemd");
			Tags = (await import("../../../lib/Tags.svelte")).default;
			Editor = bytemd.Editor;
			let response = await api.get(
				`edit/question/${id}`,
				$session.user.xsrf_token
			);

			if (response.code === 200) {
				if (response.data.title) {
					topic.title = response.data.title;
				}
				value = response.data.description;
				if (response.data.tags) {
					topic.taglist = response.data.tags.split(",");
					topic.taglist.splice(-1);
					count = topic.taglist.length;
				}
			}
			0;
		} else {
			goto("/login");
		}
	});

	function handleChange(e) {
		value = e.detail.value;
	}

	function handleTags(event) {
		topic.tagList = event.detail.tags;
		let re = /[a-zA-Z0-0\-\+]+/;
		for (let i = 0; i < topic.tagList.length; i++) {
			if (topic.tagList[i].length > 32) {
				M.toast({ html: "32 Characterx max." });
			}
		}
	}

	async function ts() {
		if ($session.user) {
			const response = await api.post(
				"get-tags",
				{
					tag: document.getElementById("tags").value,
				},
				$session.user.xsrf_token
			);
			if (response.data) {
				let tags = [];
				for (let i = 0; i < response.data.length; i++) {
					tags.push(response.tags[i]["name"]);
				}
				return tags;
			} else {
				return [];
			}
		} else {
			goto("/login");
		}
	}
</script>

<div class="topic">
	<h3>Edit your post</h3>
	<hr />
	<form on:submit|preventDefault={onSubmit}>
		{#if topic.title}
			<div class="input-field">
				<input
					bind:value={topic.title}
					label="Title"
					id="title"
					type="text"
					minlength="6"
					maxlength="256"
					style="min-width:100%"
				/>
				<label for="title">Summary</label>
			</div>
		{/if}
		<svelte:component
			this={Editor}
			on:change={handleChange}
			mode="tab"
			{value}
		/>
		<div style="margin:30px" />
		{#if count}
			<svelte:component
				this={Tags}
				name={"tags"}
				on:tags={handleTags}
				addKeys={[9]}
				maxTags={5}
				allowPaste={true}
				allowDrop={true}
				splitWith={","}
				onlyUnique={true}
				removeKeys={[27, 8]}
				placeholder={"Tags, tab to complete"}
				tags={topic.taglist}
				allowBlur={true}
				disable={false}
				id={"tags"}
				minChars={3}
				autoComplete={ts}
			/>
		{/if}
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
