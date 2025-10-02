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
	import { post } from '$lib/api.js';

	let username = '';
	let email = '';
	let errors = null;

	async function check_username_availablity() {
		if (username.length < 3) {
			return;
		} else {
			let response = await post('check-username-availability', {
				username
			});

			if (response.status !== 200) {
				response = JSON.parse(await response.text());
				errors = response.message;
			}
			response = JSON.parse(await response.text());

			if (response.message === 'username available') {
				document.getElementById('username-helper').innerHTML = 'Username available!';
				document.getElementById('username-helper').style.color = '#080';
			} else {
				document.getElementById('username-helper').innerHTML = 'Username unavailable!';
				document.getElementById('username-helper').style.color = '#800';
			}
		}
	}
	async function submit() {
		if (username.length < 3) {
			document.getElementById('username-helper').innerHTML =
				'Username must be at least three characters!';
			document.getElementById('username-helper').style.color = '#800';
			return;
		}
		const response = await post(`register`, {
			username,
			email
		});

		let text = await response.text();
		let j = text ? JSON.parse(text) : {};
		let alert = document.getElementById('registrationMessage');
		const wrapper = document.createElement('div');

		if (response.status === 200) {
			wrapper.innerHTML = [
				`<div class="alert alert-success alert-dismissible" role="alert">`,
				`   <div>An email verification email has been sent to you!</div>`,
				'   <button type="button" class="btn-close" data-bs-dismiss="alert" aria-label="Close"></button>',
				'</div>'
			].join('');
			alert.append(wrapper);
		} else {
			wrapper.innerHTML = [
				`<div class="alert alert-success alert-dismissible" role="alert">`,
				`   <div>${j.message}</div>`,
				'   <button type="button" class="btn-close" data-bs-dismiss="alert" aria-label="Close"></button>',
				'</div>'
			].join('');
			alert.append(wrapper);
		}
	}
</script>

<svelte:head>
	<title>Register ❤ Roji</title>
</svelte:head>
<div class="row justify-content-center align-items-center" style="height:80vh">
	<div class="col-4">
		<div id="registrationMessage"></div>
		<h3>Register</h3>
		<p>
			<a href="/login">Have an account?</a>
		</p>

		<form on:submit|preventDefault={submit} class="col s6">
			<div>
				<div>
					<label for="username" class="form-label">Username</label>
					<input
						class="form-control"
						type="text"
						required
						bind:value={username}
						on:keyup={check_username_availablity}
						id="username"
						placeholder="Username minimum three characters"
					/>
				</div>
				<p id="username-helper" />
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
				<p>
					We’ll never share your details. Read our <a
						href="/privacy-policy"
						class="font-medium text-primary-600 hover:underline dark:text-primary-500"
					>
						Privacy Policy
					</a>
					.
				</p>
			</div>
			<div class="b-wrapper">
				<button class="btn btn-primary" type="submit" name="action">Submit</button>
			</div>
		</form>
	</div>
</div>
