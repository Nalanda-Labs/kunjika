<script>
	import { page, session } from "$app/stores";
	import { post } from "$lib/utils.js";
	import { goto } from "$app/navigation";
	import { onMount } from "svelte";

	function logout() {
		post("/logout");
		$session.user = null;
		goto("/questions");
	}
</script>

<header>
	<nav class="top-nav">
		<div class="container">
			<div class="nav-wrapper">
				<div class="row">
					<a
						href="#"
						data-target="slide-out"
						class="top-nav sidenav-trigger full hide-on-large-only"
						><i class="material-icons">menu</i></a
					>
					<a href="/" class="brand-logo">Kunjika</a>
					<ul id="nav-mobile" class="right">
						{#if $session.user}
							<!-- <li>
								<a
									rel="prefetch"
									href="/settings"
									class:active={$page.url.pathname ===
										"/settings"}
								>
									<i class="icon-gear-a" />Settings
								</a>
							</li> -->

							<li>
								<a
									rel="prefetch"
									href="/user/{$session.user.id}/{$session
										.user.username}"
								>
									{$session.user.username}
								</a>
							</li>
							<li>
								<a rel="prefetch" href="#" on:click={logout}>
									Logout
								</a>
							</li>
						{:else}
							<li>
								<a
									rel="prefetch"
									href="/login"
									class:active={$page.url.pathname ===
										"/login"}
								>
									Login
								</a>
							</li>

							<li>
								<a
									rel="prefetch"
									href="/register"
									class:active={$page.url.pathname ===
										"/register"}
								>
									Register
								</a>
							</li>
						{/if}
					</ul>
				</div>
			</div>
		</div>
	</nav>
	<div class="container" />
	<ul
		class="sidenav sidenav-fixed"
		id="slide-out"
		style="transform: translateX(0%);"
	>
		<li class="logo">
			<a id="logo-container" href="/questions" class="brand-logo">
				Kunjika</a
			>
		</li>
		<li><div class="divider" /></li>
		<li>
			<a href="/questions">Questions</a>
		</li>
		<li><a href="/tags">Tags</a></li>
		<li><a href="/users">Users</a></li>
		<li />
	</ul>
</header>
