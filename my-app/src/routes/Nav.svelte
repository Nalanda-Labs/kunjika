<script>
	import { page } from '$app/stores';
	import { Navbar, NavBrand, A } from 'flowbite-svelte';
	import { enhance } from '$app/forms';
</script>

<Navbar color="primary">
	<NavBrand href="/">
		<img src="/logo.png" class="me-3 h-6 sm:h-9" alt="Kunjika" />
		<span class="self-center whitespace-nowrap text-xl font-semibold dark:text-white">Kunjika</span>
	</NavBrand>
	<ul
		class="font-medium flex flex-col p-4 md:p-0 mt-4 border border-gray-100 rounded-lg bg-gray-50 md:flex-row md:space-x-8 md:mt-0 md:border-0 md:bg-white dark:bg-gray-800 md:dark:bg-gray-900 dark:border-gray-700"
	>
		{#if $page.data.user}
			<li>
				<button
					id="mega-menu-dropdown-button"
					data-dropdown-toggle="mega-menu-dropdown"
					class="flex items-center justify-between w-full py-2 px-3 font-medium text-gray-900 border-b border-gray-100 md:w-auto hover:bg-gray-50 md:hover:bg-transparent md:border-0 md:hover:text-blue-600 md:p-0 dark:text-white md:dark:hover:text-blue-500 dark:hover:bg-gray-700 dark:hover:text-blue-500 md:dark:hover:bg-transparent dark:border-gray-700"
				>
					{$page.data.user.username}
					<svg
						class="w-2.5 h-2.5 ms-3"
						aria-hidden="true"
						xmlns="http://www.w3.org/2000/svg"
						fill="none"
						viewBox="0 0 10 6"
					>
						<path
							stroke="currentColor"
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="m1 1 4 4 4-4"
						/>
					</svg>
				</button>
				<div
					id="mega-menu-dropdown"
					class="absolute z-10 grid hidden w-auto grid-cols-2 text-sm bg-white border border-gray-100 rounded-lg shadow-md dark:border-gray-700 md:grid-cols-3 dark:bg-gray-700"
				>
					<div class="p-4 pb-0 text-gray-900 md:pb-4 dark:text-white">
						<ul class="space-y-4" aria-labelledby="mega-menu-dropdown-button">
							<li>
								<a
									href="/user/{$page.data.user.id}"
									class="text-gray-500 dark:text-gray-400 hover:text-blue-600 dark:hover:text-blue-500"
								>
									Profile
								</a>
							</li>
							<li>
								<form
									use:enhance
									class="text-gray-500 dark:text-gray-400 hover:text-blue-600 dark:hover:text-blue-500"
									style="margin-left:0px"
									method="POST"
									action="/logout"
								>
									<button type="submit">Sign Out</button>
								</form>
							</li>
						</ul>
					</div>
				</div>
			</li>
		{:else}
			<li>
				<A href="/register">Register</A>
			</li>
			<li>
				<A href="/login">Login</A>
			</li>
		{/if}
	</ul>
</Navbar>
