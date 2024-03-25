<script>
	import { page } from "$app/stores";
	import { enhance } from '$app/forms';

</script>

<header>
	<nav class="top-nav light-blue darken-2">
		<div class="container">
			<div class="nav-wrapper">
				<div class="row">
					<a
						href="#"
						data-target="slide-out"
						class="top-nav sidenav-trigger full hide-on-large-only"
						><i class="material-icons green">menu</i></a
					>
					<a href="/" class="brand-logo">Kunjika</a>
					<ul id="nav-mobile" class="right">
						{#if $page.data.user}
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
									href="/user/{$page.data.user.username}"
								>
									{$page.data.user.username}
								</a>
							</li>
							<li>
								<form use:enhance method="POST" action="/logout" style="margin-left:20px">
									<button type="submit" class="light-blue darken-2" style="background-color:#0288d1!important">Sign Out</button>
								</form>
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
