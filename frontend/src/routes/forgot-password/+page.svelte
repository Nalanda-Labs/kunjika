<script context="module">
	export async function load({ page }) {
		if (page.data.user) {
			return {
				status: 302,
				redirect: '/questions'
			};
		}

		return {};
	}
</script>

<script>
	import * as api from '$lib/api.js';

	let message = '';
	let email = '';

	async function submit() {
		let response = await api.post(`forgot-password/`, { email });

		let alert = document.getElementById('message');
		const wrapper = document.createElement('div');

		if (response.status === 200) {
			response = await response.text();
			response = response ? JSON.parse(response) : {};

			if (response.success) {
				message = response.message;
				wrapper.innerHTML = [
					`<div class="alert alert-success alert-dismissible" role="alert">`,
					`   <div>${message}</div>`,
					'   <button type="button" class="btn-close" data-bs-dismiss="alert" aria-label="Close"></button>',
					'</div>'
				].join('');
			} else {
				message = response.message;
				wrapper.innerHTML = [
					`<div class="alert alert-success alert-dismissible" role="alert">`,
					`   <div>${message}</div>`,
					'   <button type="button" class="btn-close" data-bs-dismiss="alert" aria-label="Close"></button>',
					'</div>'
				].join('');
			}
		} else {
			response = await response.text();
			response = response ? JSON.parse(response) : {};
			message = response.message;
			wrapper.innerHTML = [
				`<div class="alert alert-danger alert-dismissible" role="alert">`,
				`   <div>${message}</div>`,
				'   <button type="button" class="btn-close" data-bs-dismiss="alert" aria-label="Close"></button>',
				'</div>'
			].join('');
		}
		alert.append(wrapper);
	}
</script>

<svelte:head>
	<title>Forgot Password ❤ Kunjika</title>
</svelte:head>

<div class="row justify-content-center align-items-center" style="height:80vh">
	<div class="col-4">
		<div id="message"></div>
		<h3>Forgot Password</h3>
		<p>
			<a href="/register">Need an account?</a>
		</p>
		<form on:submit|preventDefault={submit} class="col s6">
			<div>
				<div>
					<label for="email" class="form-label">Email</label>
					<input
						class="form-control"
						type="email"
						required
						bind:value={email}
						minlength="6"
						maxlength="256"
					/>
				</div>
			</div>
			<div class="b-wrapper">
				<button class="btn btn-primary" type="submit" name="action">Forgot Password</button>
			</div>
		</form>
	</div>
</div>
