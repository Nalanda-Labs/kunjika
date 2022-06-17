<script>
	import { goto, stores } from "@sapper/app";
	import * as api from "api.js";
	import { onMount } from "svelte";
	import "bytemd/dist/index.min.css";
	import "../../../_utils.scss";
	import Textfield from "@smui/textfield";
	import Button, { Label } from "@smui/button";
	import HelperText from "@smui/textfield/helper-text/index";
	import Swal from 'sweetalert2';
	// import gfm from "@bytemd/plugin-gfm";

	export let tag;
    export let info;

	let inProgress = false;
	const { session } = stores();
	let Editor = null;
	let plugins = null;
	let value = "";

	async function onSubmit() {
		if ($session.user) {
			if (value.length < 20 || value.length > 100000) {
				Swal.fire(
					"Info should not be less than 20 or more than 100000 characters."
				);
				return;
			}

			info = value;
			const response = await api.post(
				`tags/edit/${tag}/`,
				{ info: info },
				localStorage.getItem("jwt")
			);

			if(response.error) {
				Swal.fire(response.error);
			}
            await goto(`/tags/${tag}`);
		} else {
			Swal.fire("You are not logged in.");
		}
	}


	onMount(async () => {
		const bytemd = await import("bytemd");
		Editor = bytemd.Editor;
        let response = await api.get(
            `tags/edit/${tag}`
        );

        if (response.info) {
            value = response.info;
        }
	});

	function handleChange(e) {
		value = e.detail.value;
	}
</script>

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

<div class="topic">
	<h3>About tag {tag}</h3>
	<hr />
	<form on:submit|preventDefault={onSubmit}>
		<svelte:component
			this={Editor}
			on:change={handleChange}
			mode="tab"
			{value}/>
		<div style="margin:30px"/>
		<div class="b-wrapper">
			<Button variant="raised">
				<Label>Save</Label>
			</Button>
		</div>
	</form>
</div>
