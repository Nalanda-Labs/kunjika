<script>
	import { page } from '$app/stores';
</script>

<header>
	<nav class="top-nav">
		<div class="container">
			<div class="nav-wrapper">
				<div class="row">
					<div class="col s12 m10 offset-m2 xl10 offset-xl2">
						<a
							href="#"
							data-target="slide-out"
							class="top-nav sidenav-trigger full hide-on-large-only"
							><i class="material-icons green">menu</i></a
						>
						<a href="/" class="brand-logo">Kunjika</a>
						<ul id="nav-mobile" class="right">
							{#if $page.data.user}
								<li>
									<a rel="prefetch" class="nav-anchor" href="/users/{$page.data.user.id}/{$page.data.user.username}" style="padding:0px;margin-right:10px">
										<img src="{$page.data.user.image_url}?s=48" alt="profile" height="48px" style="margin-top:8px"/>
									</a>
								</li>
								<li>
									<form style="margin-left:0px" method="POST" action="/logout">
										<button class="btn" type="submit">Sign Out</button>
									</form>
								</li>
							{:else}
								<li>
									<a rel="prefetch" href="/login" class:active={$page.url.pathname === '/login'}>
										Login
									</a>
								</li>

								<li>
									<a
										rel="prefetch"
										href="/register"
										class:active={$page.url.pathname === '/register'}
									>
										Register
									</a>
								</li>
							{/if}
						</ul>
					</div>
				</div>
			</div>
		</div>
	</nav>
	<!-- <div class="container" /> -->
	<ul class="sidenav sidenav-fixed" id="slide-out" style="transform: translateX(0%);">
		<li class="logo">
			<a id="logo-container" href="/questions" class="brand-logo"> Kunjika</a>
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
