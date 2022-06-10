<script>
	import { page, session } from "$app/stores";
</script>

<!--nav class="navbar navbar-light">
	<div class="container">
		<a rel="prefetch" class="navbar-brand" href="/">conduit</a>
		<ul class="nav navbar-nav pull-xs-right">
			<li class="nav-item">
				<a rel="prefetch" class="nav-link" class:active={$page.url.pathname === '/'} href="/">Home</a>
			</li>

			{#if $session.user}
				<li class="nav-item">
					<a rel="prefetch" href="/editor" class="nav-link" class:active={$page.url.pathname === '/editor'}>
						<i class="ion-compose" />&nbsp;New Post
					</a>
				</li>

				<li class="nav-item">
					<a
						rel="prefetch"
						href="/settings"
						class="nav-link"
						class:active={$page.url.pathname === '/settings'}>
						<i class="ion-gear-a" />&nbsp;Settings
					</a>
				</li>

				<li class="nav-item">
					<a rel="prefetch" href="/profile/@{$session.user.username}" class="nav-link">
						< <img src={$user.image} class="user-pic" alt={$user.username}> >
						{$session.user.username}
					</a>
				</li>
			{:else}
				<li class="nav-item">
					<a rel="prefetch" href="/login" class="nav-link" class:active={$page.url.pathname === '/login'}>
						Sign in
					</a>
				</li>

				<li class="nav-item">
					<a
						rel="prefetch"
						href="/register"
						class="nav-link"
						class:active={$page.url.pathname === '/register'}>
						Sign up
					</a>
				</li>
			{/if}
		</ul>
	</div>
</nav-->
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
							<li>
								<a
									rel="prefetch"
									href="/settings"
									class:active={$page.url.pathname ===
										"/settings"}
								>
									<i class="ion-gear-a" />&nbsp;Settings
								</a>
							</li>

							<li>
								<a
									rel="prefetch"
									href="/profile/@{$session.user.username}"
								>
									{$session.user.username}
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
			<a id="logo-container" href="/" class="brand-logo"> Kunjika</a>
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
