<script>
	export let errors;
</script>

{#if errors}
	<p class="error-messages">
		<!-- {#each Object.keys(errors) as key} -->
			<!-- <li>{key} {errors[key]}</li> -->
		<!-- {/each} -->
		{errors}
	</p>
{/if}