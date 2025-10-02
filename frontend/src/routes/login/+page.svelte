<script>
	import { enhance, applyAction } from '$app/forms';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import ListErrors from '$lib/ListErrors.svelte';
	import * as api from '$lib/api.js';

	/** @type {import('./$types').ActionData} */
	export let form;
	let email;

	let errors = null;
	let step = 1;

	if ($page.data.user) {
		goto('/questions');
	}

	async function onEmailSubmit() {
		const response = await api.post('submit-email', { email: email });

		if (response.status === 200) {
			step = 2;
		} else {
			alert('Something went wrong! Try again or contact support!');
		}
	}
</script>

<svelte:head>
	<title>Login</title>
</svelte:head>

<div class="row justify-content-center align-items-center" style="height:80vh">
	{#if step === 1}
		<div class="col-4">
			<h3>Login</h3>
			<br />
			<p>
				<a href="/register">Need an account?</a>
			</p>

			<ListErrors errors={form?.errors} />
			<form method="post" on:submit|preventDefault={onEmailSubmit}>
				<div>
					<div class="mb-3">
						<label for="email" class="form-label">Email</label>
						<input
							type="email"
							id="email"
							name="email"
							class="form-control"
							required
							bind:value={email}
						/>
					</div>
				</div>
				<div class="b-wrapper">
					<button class="btn btn-primary" type="submit">Submit </button>
				</div>
			</form>
		</div>
	{:else if step === 2}
		<div class="col-4">
			<h3>Submit Sign-in Code</h3>
			<br />
			<p>An email has been sent with sign-in code.</p>
			<ListErrors errors={form?.errors} />
			<form method="post" action="?/login">
				<div>
					<div class="mb-3">
						<input
							type="hidden"
							id="email"
							name="email"
							class="form-control"
							bind:value={email}
							required
						/>
					</div>
					<div class="mb-3">
						<label for="sign_in_code" class="form-label">Sign-in Code</label>
						<input
							type="text"
							id="sign-in-code"
							name="sign_in_code"
							class="form-control"
							required
						/>
					</div>
				</div>
				<div class="b-wrapper">
					<button class="btn btn-primary" type="submit">Submit </button>
				</div>
			</form>
		</div>
	{/if}
</div>
