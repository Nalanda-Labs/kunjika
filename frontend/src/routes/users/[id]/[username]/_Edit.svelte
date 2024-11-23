<script>
	import { browser } from '$app/environment';
	import * as api from '$lib/api.js';
	import getCookie from '../../../../lib/cookie.js';
	import { goto } from '$app/navigation';

	export let id;
	export let imageUrl;
	export let username;
	export let designation;
	export let git;
	export let website;
	export let location;

	async function onSubmit() {
		if (browser) {
			let xsrf_token = getCookie('xsrf_token');
			const response = await api.post(
				`${id}/update-profile`,
				{ username, designation, git, website, location },
				xsrf_token
			);

			let text = await response.text();
			let j = text ? JSON.parse(text) : {};

			if (response.status === 200 && j.success) {
				alert('Profile data saved');
			} else {
				alert(j.messages);
			}
		} else {
			throw redirect(307, '/questions');
		}
	}

	async function showProfileImageUploadForm() {
		let form = document.getElementById('profileImageUploadForm');
		form.style.display = 'block';
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
			let xsrf_token = getCookie('xsrf_token');
			const response = await api.upload({
				method: 'POST',
				path: `${id}/profile-image-upload`,
				data: formData,
				xsrf_token,
				headers: null
			});

			let text = await response.text();
			let j = text ? JSON.parse(text) : {};

			if (response.status === 200 && j.url) {
				closeForm();
				imageUrl = j.url;
			} else {
				alert(j.message);
			}
		}
	}

	function closeForm() {
		document.getElementById('profileImageUploadForm').style.display = 'none';
	}
</script>

<div class="row justify-content-center align-items-center" style="margin-top:20px">
	<div class="col-12">
		<p style="font-weight: bold;">Profile Image(Click to upload)</p>
		<a
			href="/users/{id}/{username}"
			on:click|preventDefault={showProfileImageUploadForm}
			style="display:flex;float:left"
		>
			<img
				src={imageUrl}
				alt="{username}'s avatar"
				style="display:flex;float:left;width:256px;height:256px"
				title="Change profile image"
			/>
		</a>
		<form on:submit|preventDefault={onSubmit}>
			<div>
				<div class="mb-3 row">
					<label for="displayname" class="col-sm-2 col-form-label" style="font-weight: bold;"
						>Display Name</label
					>
					<div class="col-sm-10">
						<input
							type="text"
							class="form-control"
							id="displayame"
							name="displayname"
							bind:value={username}
						/>
					</div>
				</div>
				<div class="mb-3 row">
					<label for="designation" class="col-sm-2 col-form-label" style="font-weight: bold;"
						>Designation</label
					>
					<div class="col-sm-10">
						<input
							type="text"
							class="form-control"
							id="designation"
							name="designation"
							bind:value={designation}
						/>
					</div>
				</div>
				<div class="mb-3 row">
					<label for="git" class="col-sm-2 col-form-label" style="font-weight: bold;"
						>git <i class="fa-brands fa-github" style="margin-right:5px"></i></label
					>
					<div class="col-sm-10">
						<input type="text" class="form-control" id="git" name="git" bind:value={git} />
					</div>
				</div>
				<div class="mb-3 row">
					<label for="git" class="col-sm-2 col-form-label" style="font-weight: bold;"
						>Website <i class="fa-solid fa-link"></i></label
					>
					<div class="col-sm-10">
						<input
							type="text"
							class="form-control"
							id="website"
							name="website"
							bind:value={website}
						/>
					</div>
				</div>
				<div class="mb-3 row">
					<label for="git" class="col-sm-2 col-form-label" style="font-weight: bold;"
						>Location <i class="fa-solid fa-location-dot" style="margin-left:5px"></i></label
					>
					<div class="col-sm-10">
						<input
							type="text"
							class="form-control"
							id="location"
							name="location"
							bind:value={location}
						/>
					</div>
				</div>
			</div>
			<div class="b-wrapper">
				<button type="submit" class="btn btn-primary">Submit Profile</button>
			</div>
		</form>
		<div class="modal" tabindex="-5" id="profileImageUploadForm" style="display:none">
			<div class="modal-dialog modal-dialog-centered" role="document">
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
										document.getElementById('profileImageUploadForm').style.display = 'none';
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
