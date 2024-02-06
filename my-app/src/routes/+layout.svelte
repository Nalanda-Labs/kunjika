<script>
	import { navigating } from '$app/stores';
	import Nav from './Nav.svelte';
	import PreloadingIndicator from './PreloadingIndicator.svelte';
	import Sidebar from './Sidebar.svelte';
	import { page } from '$app/stores';
	import Footer from './Footer.svelte';
	import '../app.css';
</script>

{#if $navigating}
	<PreloadingIndicator />
{/if}

<Nav />
<Sidebar />
<main>
	<slot />
</main>
<Footer />
