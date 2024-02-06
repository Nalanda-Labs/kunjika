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
	import ListErrors from '$lib/ListErrors.svelte';
	import { A, Toast, Label, Input, Button, Helper } from 'flowbite-svelte';

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

			if (response.status) {
				document.getElementById('username-helper').innerHTML= 'Username available!';
				document.getElementById('username-helper').style.color ='#080';
			} else {
				M.toast({ html: 'Username unavailable' });
			}
		}
	}
	async function submit() {
		if (username.length < 3) {
			document.getElementById('username-helper').innerHTML= 'Username must be at least three characters!';
			document.getElementById('username-helper').style.color ='#800';
			return;
		}
		if (password !== confirm_password && (password.length < 16 || confirm_password.length)) {
			document.getElementById('username-helper').innerHTML= 'Passphrases must be at least 16 characters and be same.';
			document.getElementById('username-helper').style.color ='#800';
			return;
		}
		const response = await post(`register`, {
			username,
			email,
			password,
			confirm_password
		});

		// if (response.code === 200) {
		//     Toast({
		//         html: "You have been sent a confirmation email. Please verify!",
		//     });
		// } else {
		//     Toast({
		//         html: response.message,
		//     });
		// }
	}
</script>

<svelte:head>
	<title>Register ❤ Kunjika</title>
</svelte:head>

<main>
	<div class="grid grid-cols-[repeat(auto-fit,_20.666666%)] m-auto p-24 justify-center mt-10">
		<div class="w-full p-8 col-span-2 justify-center justify-self-center mx-auto text-lg">
			<h3 class="text-xl font-bold">Register</h3>
			<p class="text-xs-center mt-2 mb-2" style="margin-left:20px">
				<A href="/login">Have an account?</A>
			</p>

			<ListErrors {errors} />

			<form on:submit|preventDefault={submit} class="col s6">
				<div class="row">
					<div class="mb-6">
						<Label for="large-input" class="block mb-2">Username</Label>
						<Input
							size="lg"
							required
							bind:value={username}
							on:keyup={check_username_availablity}
							id="username"
							placeholder="Username minimum 3 characters"
						/>
						<Helper class="text-sm mt-2" id="username-helper" />
					</div>
					<div class="mb-6">
						<Label for="large-input" class="block mb-2">Email</Label>
						<Input
							type="email"
							size="lg"
							required
							bind:value={email}
							minlength="6"
							maxlength="256"
							id="email"
							placeholder="Email"
						/>
						<Helper class="text-sm mt-2">
							We’ll never share your details. Read our <a
								href="/privacy-policy"
								class="font-medium text-primary-600 hover:underline dark:text-primary-500"
							>
								Privacy Policy
							</a>
							.
						</Helper>
					</div>
					<div class="mb-6">
						<Label for="large-input" class="block mb-2">Passphrase</Label>
						<Input
							size="lg"
							required
							bind:value={password}
							minlength="16"
							maxlength="64"
							id="email"
							placeholder="Password"
							type="password"
						/>
						<Helper class="text-sm mt-2" id="password-helper" />
					</div>
					<div class="mb-6">
						<Label for="large-input" class="block mb-2">Confirm Passphrase</Label>
						<Input
							size="lg"
							required
							bind:value={confirm_password}
							minlength="16"
							maxlength="64"
							id="email"
							placeholder="Repeat Password"
							type="password"
						/>
					</div>
					<div class="b-wrapper">
						<Button class="btn" type="submit" name="action"
							>Submit
							<i class="material-icons right">send</i>
						</Button>
					</div>
				</div>
			</form>
		</div>
	</div>
</main>
