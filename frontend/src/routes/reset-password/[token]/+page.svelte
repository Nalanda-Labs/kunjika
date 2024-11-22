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
	import { page } from '$app/stores';
	import * as api from '$lib/api.js';
	import { onMount } from 'svelte';

	let message = '';
	let correctToken = false;
	let password = '';
	let confirm_password = '';

	onMount(async () => {
		let response = await api.get(`reset-password/${encodeURI($page.params.token)}`);

		if (response.status === 200) {
			response = await response.text();
			response = response ? JSON.parse(response) : {};

			if (response.success) {
				correctToken = true;
			} else {
				message = response.message;
			}
		} else {
			message = `Something went wrong! Contact support with message "${response.message}"`;
		}
	});

	async function submit() {
		if (password !== confirm_password && (password.length < 16 || confirm_password.length)) {
			document.getElementById('password-helper').innerHTML =
				'Passphrases must be at least 16 characters and be same.';
			document.getElementById('password-helper').style.color = '#800';
			return;
		}

		let response = await api.post(`reset-password/${encodeURI($page.params.token)}`, {
			password,
			confirm_password
		});

		let alert = document.getElementById('resetMessage');
		const wrapper = document.createElement('div');

		if (response.status === 200) {
			response = await response.text();
			response = response ? JSON.parse(response) : {};
			wrapper.innerHTML = [
				`<div class="alert alert-success alert-dismissible" role="alert">`,
				`   <div>Password has been successfully reset!</div>`,
				'   <button type="button" class="btn-close" data-bs-dismiss="alert" aria-label="Close"></button>',
				'</div>'
			].join('');
			alert.append(wrapper);
		} else {
			response = await response.text();
			response = response ? JSON.parse(response) : {};
			wrapper.innerHTML = [
				`<div class="alert alert-danger alert-dismissible" role="alert">`,
				`   <div>${response.message}</div>`,
				'   <button type="button" class="btn-close" data-bs-dismiss="alert" aria-label="Close"></button>',
				'</div>'
			].join('');
			alert.append(wrapper);
		}
	}
</script>

<svelte:head>
	<title>Forgot Password ❤ Kunjika</title>
</svelte:head>

<div class="row justify-content-center align-items-center" style="height:80vh">
	<div class="col-4">
		<p>{message}</p>
		<div id="resetMessage"></div>
		{#if correctToken}
			<div id="message"></div>
			<h3>Reset Password</h3>
			<p>
				<a href="/register">Need an account?</a>
			</p>
			<form on:submit|preventDefault={submit} class="col s6">
				<div>
					<div>
						<label for="password" class="form-label">Passphrase</label>
						<input
							class="form-control"
							type="password"
							required
							bind:value={password}
							minlength="16"
							maxlength="64"
							placeholder="minimum 16 characters"
						/>
					</div>
					<label for="password" class="form-label">Confirm Passphrase</label>
					<div class="input-field col s12">
						<input
							class="form-control"
							type="password"
							required
							bind:value={confirm_password}
							minlength="16"
							maxlength="64"
							placeholder="Repeat passphrase"
						/>
					</div>
					<p id="password-helper"></p>
				</div>
				<div class="b-wrapper">
					<button class="btn btn-primary" type="submit" name="action">Reset Password</button>
				</div>
			</form>
		{/if}
	</div>
</div>
