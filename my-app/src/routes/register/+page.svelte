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
	let password = '';
	let confirm_password = '';
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

			if (response.status === 200) {
				document.getElementById('username-helper').innerHTML = 'Username available!';
				document.getElementById('username-helper').style.color = '#080';
			} else {
				M.toast({ html: 'Username unavailable' });
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
		if (password !== confirm_password && (password.length < 16 || confirm_password.length)) {
			document.getElementById('username-helper').innerHTML =
				'Passphrases must be at least 16 characters and be same.';
			document.getElementById('username-helper').style.color = '#800';
			return;
		}
		const response = await post(`register`, {
			username,
			email,
			password,
			confirm_password
		});

		let text = await response.text();
		let j = text ? JSON.parse(text) : {};
		if (response.status === 200) {
		    M.toast({
		        html: "You have been sent a confirmation email. Please verify!",
		    });
		} else {
		    M.toast({
		        html: j.message,
		    });
		}
	}
</script>

<svelte:head>
	<title>Register ❤ Kunjika</title>
</svelte:head>
<main>
	<div class="row">
		<div class="col-md-6 offset-md-3 col-xs-12">
			<h3 class="text-xs-center">Register</h3>
			<p class="text-xs-center" style="margin-left:20px">
				<a href="/login">Have an account?</a>
			</p>

			<form on:submit|preventDefault={submit} class="col s6">
				<div class="row">
					<div class="input-field col s12">
						<input
							class="validate"
							type="text"
							required
							bind:value={username}
							on:keyup={check_username_availablity}
							id="username"
							placeholder="Username minimum three characters"
						/>
						<label for="username">Username</label>
					</div>
					<p id="username-helper" />
					<div class="input-field col s12">
						<input
							class="validate"
							type="email"
							required
							bind:value={email}
							minlength="6"
							maxlength="256"
						/>
						<label for="email">Email</label>
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
					<div class="input-field col s12">
						<input
							class="validate"
							type="password"
							required
							bind:value={password}
							minlength="16"
							maxlength="64"
							placeholder="minimum 16 characters"
						/>
						<label for="password">Passphrase</label>
					</div>
					<div class="input-field col s12">
						<input
							class="validate"
							type="password"
							required
							bind:value={confirm_password}
							minlength="16"
							maxlength="64"
							placeholder="Repeat passphrase"
						/>
						<label for="password">Confirm Passphrase</label>
					</div>
				</div>
				<div class="b-wrapper">
					<button class="btn" type="submit" name="action"
						>Submit
						<i class="material-icons right">send</i>
					</button>
				</div>
			</form>
		</div>
	</div>
</main>
