<script lang="ts">
    import { page } from '$app/state';
</script>

<svelte:head>
    <title>{page.status}</title>
</svelte:head>

<h1 class="my-16 text-center text-4xl font-light sm:text-6xl">
    {page.status === 404 ? 'Not found!' : page.error?.message}
</h1>
