<script>
	import { goto } from '$app/navigation';
	import * as api from '$lib/api.js';
	import { page } from '$app/stores';
	import { browser } from '$app/environment';
	import Tags from 'svelte-tags-input';
	import '../highlight.css';
	import { redirect } from '@sveltejs/kit';
	import Editor from '../../components/Editor/Editor.svelte';
	import Preview from '../../components/Editor/Preview.svelte';
	import getCookie from '../../lib/cookie.js';
	import '../../editor.css';
	import onImageUpload from '../../lib/imageUpload';
	import { firstDuplicateTag } from '../../lib/utils/tags.js';

	let title = '';
	let tagList = [];
	let contentValue = '',
		markup = '';
	let formError = '';

	async function onSubmit() {
		if ($page.data.user) {
			if (contentValue.length < 20 || contentValue.length > 100000) {
				formError = 'Question length should be 20 to 1000000 Characters!';
				return;
			}
			if (tagList.length < 1) {
				formError = 'At least one tag should be supplied.';
				return;
			}
			if (tagList.length > 5) {
				formError = 'Too many tags.';
				return;
			}
			const duplicate = firstDuplicateTag(tagList);
			if (duplicate) {
				formError = `Duplicate tag: ${duplicate}`;
				return;
			}
			formError = '';

			if (browser) {
				let xsrf_token = getCookie('xsrf_token');
				const response = await api.post(
					'create-question',
					{ title: title, description: contentValue, tag_list: tagList },
					xsrf_token
				);

				let text = await response.text();
				let j = text ? JSON.parse(text) : {};

				if (response.status === 200 && j.data.id) {
					let id = j.data.id;
					let slug = j.data.slug;
					await goto(`/questions/${id}/${slug}`);
				} else {
					// Was `response.code === 400`, which is always undefined on a
					// Response - so backend errors were silently swallowed. And
					// `M.toast` is not defined anywhere in the app, so it threw
					// a ReferenceError instead of showing anything.
					formError = j.message || 'Something went wrong, try again!';
				}
			}
		} else {
			throw redirect(307, '/questions');
		}
	}

	// function for auto-completing tags
	async function ts() {
		if (browser) {
			let xsrf_token = getCookie('xsrf_token');
			let response = await api.post(
				'get-tags',
				{
					tag: document.getElementById('tags').value
				},
				xsrf_token
			);
			response = JSON.parse(await response.text());
			if (response.data) {
				let tags = [];
				for (let i = 0; i < response.data.length; i++) {
					tags.push(response.data[i].name);
				}
				return tags;
			} else {
				return [];
			}
		}
	}
</script>

<div class="row justify-content-center align-items-center" style="margin-top:20px">
	<div class="col-12">
		{#if formError}
			<div class="alert alert-danger" role="alert" id="askError">{formError}</div>
		{/if}
		<h4>Post your question for discussion</h4>
		<hr />
		<form on:submit|preventDefault={onSubmit}>
			<div>
				<label for="title" class="form-label">Summary</label>
				<input
					bind:value={title}
					label="Title"
					class="form-control"
					id="title"
					type="text"
					minlength="6"
					maxlength="256"
					style="min-width:100%"
				/>
			</div>
			<Editor bind:markup bind:contentValue minlength={20} maxlength={100000} />
			<div style="margin:20px"></div>
			<Preview {markup} />
			<div style="margin:30px"></div>
			<Tags
				name={'tags'}
				bind:tags={tagList}
				addKeys={[9]}
				maxTags={5}
				allowPaste={true}
				allowDrop={true}
				splitWith={','}
				onlyUnique={true}
				removeKeys={[27, 8]}
				placeholder="Tags, tab to complete"
				allowBlur={true}
				disable={false}
				id={'tags'}
				minChars={1}
				autoComplete={ts}
				class="form-control"
			/>
			<div class="b-wrapper">
				<button type="submit" class="btn btn-primary"> Ask </button>
			</div>
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
</div>
