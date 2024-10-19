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
	import { closeForm, addImageURL } from '../../lib/utils/editor/utils.editor';

	let title = '';
	let tagList = [];
	let contentValue = '',
		markup = '';
	let showContentValueToast = false;

	async function onSubmit() {
		if ($page.data.user) {
			if (contentValue.length < 20 || contentValue.length > 100000) {
				showContentValueToast = true;
			}
			if (tagList.length < 1) {
				// Toast('At least one tag should be supplied.');
			}

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
				} else if (response.code === 400) {
					M.toast({ html: j.message });
				}
			}
		} else {
			throw redirect(307, '/questions');
		}
	}

	// function handleTags(event) {
	// 	tagList = event.detail.tags;
	// 	console.log(tagList);
	// 	let re = /[a-zA-Z0-0\-\+]+/;
	// 	for (let i = 0; i < tagList.length; i++) {
	// 		if (tagList[i].length > 32) {
	// 			document.getElementById('tags-helper').innerHTML = '32 Characterx max.';
	// 			document.getElementById('tags-helper').style.color = '#800';
	// 			break;
	// 		} else {
	// 			document.getElementById('tags-helper').innerHTML = '';
	// 		}
	// 	}
	// }

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

	async function onImageUpload(e) {
		e.preventDefault();
		let formData = new FormData();
		let image = document.getElementById('image').files[0];

		if (!image) {
			alert('No image selected!');
		}

		if (image.size > 2 * 1024 * 1024) {
			alert('Max file size is 2MB');
			return;
		}

		formData.append('file', image);

		if (browser) {
			console.log('hello');
			let xsrf_token = getCookie('xsrf_token');
			const response = await api.upload({
				method: 'POST',
				path: 'image-upload',
				data: formData,
				xsrf_token,
				headers: null
			});

			let text = await response.text();
			let j = text ? JSON.parse(text) : {};
			console.log(j);

			if (response.status === 200 && j.url) {
				closeForm();
				addImageURL(`![alt](${j.url})`);
			} else {
				alert(j.message);
			}
		}
	}

	closeForm = () => {
		document.getElementById('myForm').style.display = 'none';
	};
</script>

<div class="row justify-content-center align-items-center" style="margin-top:20px">
	<div class="col-8">
		{#if showContentValueToast}
			<Toast>Question length should be 20 to 1000000 Characters!</Toast>
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
			<div style="margin:20px" />
			<Preview {markup} />
			<div style="margin:30px" />
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
	</div>
</div>
