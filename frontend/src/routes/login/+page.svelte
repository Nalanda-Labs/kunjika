<script>
	import { enhance, applyAction } from '$app/forms';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import ListErrors from '$lib/ListErrors.svelte';

	let errors = null;

	if ($page.data.user) {
		goto('/questions');
	}

	/** @type {import('./$types').ActionData} */
	export let form;
</script>

<svelte:head>
	<title>Login</title>
</svelte:head>

<div class="row justify-content-center align-items-center" style="height:80vh">
	<div class="col-4">
	<h3>Login</h3>
	<br />
	<p>
		<a href="/register">Need an account?</a>
	</p>

	<ListErrors errors={form?.errors} />
	<form method="post" action="?/login">
		<div>
			<div class="mb-3">
				<label for="email" class="form-label">Email</label>
				<input
					type="email"
					id="email"
					name="email"
					class="form-control"
					required
					value={form?.email ?? ''}
				/>
			</div>
			<div class="">
				<label for="password" class="form-label">Passsword (<a href="/forgot-password">Forgot Password</a>)</label>
				<input
					class="form-control"
					type="password"
					id="password"
					name="password"
					required
					minlength="16"
				/>
			</div>
		</div>
		<div class="b-wrapper">
			<button class="btn btn-primary" type="submit">Submit </button>
		</div>
	</form>
</div>
</div>