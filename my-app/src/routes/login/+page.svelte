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

<div class="row">
	<div class="col s12 m12 offset-m4 xl12 offset-xl4">
		<h3 class="text-xs-center">Login</h3>
		<p class="text-xs-center">
			<a href="/register">Need an account?</a>
		</p>

		<ListErrors errors={form?.errors} />
		<form class="col s6" method="post" action="?/login">
			<div class="row">
				<div class="input-field col s12">
					<input
						class="validate"
						type="email"
						id="email"
						name="email"
						required
						value={form?.email ?? ''}
					/>
					<label for="email">Email</label>
				</div>
				<div class="input-field col s12">
					<input
						class="validate"
						type="password"
						id="password"
						name="password"
						required
						minlength="16"
					/>
					<label for="password">Passsword</label>
				</div>
			</div>
			<div class="b-wrapper">
				<button class="btn light-blue darken-2" type="submit"
					>Submit
					<i class="material-icons right">send</i>
				</button>
			</div>
		</form>
	</div>
</div>
