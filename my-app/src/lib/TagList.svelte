<script>
	export let taglist;
</script>

<ul style="list-style-type:none;padding:0px;display:inline">
	{#each taglist as tag}
		<li style="display:inline">
			<a
				href="/tags/questions/tagged/{tag}"
				class="light-blue darken-2"
				style="display:inline;padding:5px;border-radius:3px;text-decoration:none; color: #fff; margin-right:10px; border-radius: 4px"
				>{tag}</a
			>
		</li>
	{/each}
</ul>
