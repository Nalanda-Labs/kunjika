<script>
	import { page } from '$app/stores';
	import { browser } from '$app/environment';

	if (browser) {
		(() => {
			'use strict';
			const tooltipTriggerList = Array.from(
				document.querySelectorAll('[data-bs-toggle="tooltip"]')
			);
			tooltipTriggerList.forEach((tooltipTriggerEl) => {
				new bootstrap.Tooltip(tooltipTriggerEl);
			});
		})();
	}
</script>

<nav class="navbar navbar-expand-lg bg-body-tertiary">
	<div class="container">
		<a class="navbar-brand" href="/">Kunjika</a>
		<button
			class="navbar-toggler"
			type="button"
			data-bs-toggle="collapse"
			data-bs-target="#navbarSupportedContent"
			aria-controls="navbarSupportedContent"
			aria-expanded="false"
			aria-label="Toggle navigation"
		>
			<span class="navbar-toggler-icon"></span>
		</button>
		<div class="collapse navbar-collapse" id="navbarSupportedContent">
			<ul class="navbar-nav me-auto mb-2 mb-lg-0">
				<li class="nav-item">
					<a rel="prefetch" class="nav-link" class:active={$page.url.pathname.startsWith('/questions')} href="/questions">Questions </a>
				</li>
				<li class="nav-item">
					<a rel="prefetch" class="nav-link" class:active={$page.url.pathname.startsWith('/tags')} href="/tags">Tags </a>
				</li>
				<li class="nav-item">
					<a rel="prefetch" class="nav-link" class:active={$page.url.pathname.startsWith('/users')} href="/users">Users {$page.path}</a>
				</li>
				<li class="nav-item">
					<a class="nav-link" href="https://github.com/Nalanda-Labs/kunjika" target="_blank" aria-label="github"
						><i class="fa-brands fa-github"></i>
					</a>
				</li>
			</ul>
			<ul class="navbar-nav ms-auto mb-2 mb-lg-0">
				{#if $page.data.user}
					<li class="nav-item">
						<a
							rel="prefetch"
							class="nav-link"
							href="/users/{$page.data.user.id}/{$page.data.user.username}"
						>
							<img src={$page.data.user.image_url} alt="profile" style="width:32px;height:32px" />
						</a>
					</li>
					<li class="nav-item">
						<form style="margin-top:5px;" method="POST" action="/logout">
							<button class="btn" style="color:#fff" type="submit">Sign Out</button>
						</form>
					</li>
				{:else}
					<li class="nav-item">
						<a rel="prefetch" href="/login" class="nav-link"> Login </a>
					</li>

					<li class="nav-item">
						<a rel="prefetch" href="/register" class="nav-link"> Register </a>
					</li>
				{/if}
			</ul>
		</div>
	</div>
</nav>
