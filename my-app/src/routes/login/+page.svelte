<script>
	import { enhance } from '$app/forms';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';

	if ($page.data.user) {
		goto('/questions');
	}

	/** @type {import('./$types').ActionData} */
	export let form;
</script>

<svelte:head>
	<title>Login</title>
</svelte:head>

<div class="home">
	{#if !$page.data.user}
		<section class="">
			<div class="flex flex-col items-center justify-center px-6 py-8 mx-auto lg:py-0" style="height:70vh">
				<div
					class="w-full bg-white rounded-lg shadow dark:border md:mt-0 sm:max-w-md xl:p-0 dark:bg-gray-800 dark:border-gray-700"
				>
					<div class="p-6 space-y-4 md:space-y-6 sm:p-8">
						<h1
							class="text-xl font-bold leading-tight tracking-tight text-gray-900 md:text-2xl dark:text-white"
						>
							Sign in to your account
						</h1>
						<form class="space-y-4 md:space-y-6" use:enhance method="post" action="?/login">
							<div>
								<label
									for="email"
									class="block mb-2 text-sm font-medium text-gray-900 dark:text-white"
									>Your email</label
								>
								<input
									type="email"
									name="email"
									id="email"
									minlength="3"
									maxlength="255"
									class="bg-gray-50 border border-gray-300 text-gray-900 sm:text-sm rounded-lg focus:ring-primary-600 focus:border-primary-600 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500"
									placeholder="name@company.com"
									value={form?.email||''}
									required
								/>
							</div>
							<div>
								<label
									for="password"
									class="block mb-2 text-sm font-medium text-gray-900 dark:text-white"
									>Password</label
								>
								<input
									type="password"
									name="password"
									id="password"
									placeholder="••••••••"
									minlength="16"
									maxlength="64"
									class="bg-gray-50 border border-gray-300 text-gray-900 sm:text-sm rounded-lg focus:ring-primary-600 focus:border-primary-600 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500"
									required
									value={form?.password||''}
								/>
							</div>
							{#if form?.success == false}
							<span class="text-red-500 text-xs">Username or password is wrong!</span>
							{/if}
							<div class="flex items-center justify-between">
								<a
									href="/forgot-password"
									class="text-sm font-medium text-primary-600 hover:underline dark:text-primary-500"
									>Forgot password?</a
								>
							</div>
							<button
								type="submit"
								class="w-full bg-primary-600 hover:bg-primary-700 text-white font-medium rounded-lg text-sm px-5 py-2.5 text-center"
								>Sign in</button
							>
						</form>
					</div>
				</div>
			</div>
		</section>
		<footer class="bg-white md:p-8 lg:p-10 dark:bg-gray-800">
			<div class="mx-auto max-w-screen-xl text-center">
				<ul class="flex flex-wrap justify-center items-center text-gray-900 dark:text-white">
					<li>
						<a href="/about" class="mr-4 hover:underline md:mr-6">About</a>
					</li>
					<li>
						<a href="/faqs" class="mr-4 hover:underline md:mr-6">FAQs</a>
					</li>
					<li>
						<a href="/contact" class="mr-4 hover:underline md:mr-6">Contact</a>
					</li>
				</ul>
			</div>
		</footer>
	{/if}
</div>
