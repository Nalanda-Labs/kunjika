<script>
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import * as api from '$lib/api.js';
	import { onMount } from 'svelte';
	import { browser } from '$app/environment';
	import Editor from '../../../../components/Editor/Editor.svelte';
	import Preview from '../../../../components/Editor/Preview.svelte';
	import getCookie from '../../../../lib/cookie.js';
	import onImageUpload from '../../../../lib/imageUpload';
    import '../../../../editor.css';

	let markup = '';
	let contentValue = '';

	onMount(async () => {
		let response = await api.get(`get-tag-info/${encodeURIComponent($page.params.tag)}`);

		if (response.status === 200) {
			response = JSON.parse(await response.text());
			contentValue = response.info;
		} else {
			response = JSON.parse(await response.text());
			alert(response.message);
		}
	});
	async function updateTagInfo() {
        if ($page.data.user) {
			if (contentValue.length < 200 || contentValue.length > 100000) {
				showContentValueToast = true;
			}

			if (browser) {
				let xsrf_token = getCookie('xsrf_token');
				const response = await api.post(
					`tags/edit/${encodeURIComponent($page.params.tag)}`,
					{ tag_info: contentValue },
					xsrf_token
				);

				let text = await response.text();
				let j = text ? JSON.parse(text) : {};

				if (response.status === 200) {
					await goto(`/tags/info/${$page.params.tag}`);
				} else  {
					alert( j.message );
				}
			}
		} else {
			throw redirect(307, '/questions');
		}
    }
</script>

<svelte:head>
	<title>Edit Tag Info ❤ Kunjika</title>
</svelte:head>

<div style="margin-top:20px; max-width:800px;margin:auto">
	<h4 class="text-xl font-bold">{$page.params.tag} Info</h4>
	<hr />
	<form on:submit|preventDefault={updateTagInfo}>
		<Editor bind:markup bind:contentValue minlength={200} maxlength={100000} />
		<div style="margin:20px"></div>
		<Preview {markup} />
		<div class="b-wrapper">
			<button class="btn btn-primary">Update Tag Info</button>
		</div>
		<div style="min-height: 20px;"></div>
	</form>
	<div
		class="modal modal-dialog-centered"
		tabindex="-1"
		role="dialog"
		id="myForm"
		style="top:300px; display:none"
	>
		<div class="modal-dialog" role="document">
			<div class="modal-content">
				<div class="modal-header">
					<h5 class="modal-title">Image Upload</h5>
				</div>
				<div class="modal-body">
					<form class="form-container" on:submit|preventDefault={onImageUpload}>
						<h4>Uplaoad Image(Max 2MB)</h4>
						<input type="file" name="image" accept="image/*" id="image" alt="image" /><br />

						<div class="modal-footer">
							<button type="submit" class="btn btn-primary">Upload</button>
							<button
								type="button"
								class="btn btn-primary"
								on:click={() => {
									document.getElementById('myForm').style.display = 'none';
								}}>Close</button
							>
						</div>
					</form>
				</div>
			</div>
		</div>
	</div>
</div>
