<script>
	import { onMount } from "svelte";
	import { goto } from "$app/navigation";

	onMount(async () => {
		goto("/questions");
	});
</script>
