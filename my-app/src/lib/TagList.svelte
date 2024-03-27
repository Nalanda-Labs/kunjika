<script>
	export let taglist;
</script>

<ul style="list-style-type:none;padding:0px;display:inline">
	{#each taglist as tag}
		<li style="display:inline">
			<a
				href="/tags/questions/tagged/{tag}"
				class="light-blue darken-2"
				style="display: inline; padding: 5px; border-radius: 3px; text-decoration: none; background-color: rgb(240, 240, 255); margin-right: 10px; font-size: 12px;"
				>{tag}</a
			>
		</li>
	{/each}
</ul>
