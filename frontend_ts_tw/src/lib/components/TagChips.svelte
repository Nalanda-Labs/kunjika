<script lang="ts">
    let { tags }: { tags: string[] } = $props();
</script>

<ul class="inline-flex flex-wrap gap-2">
    {#each tags.filter((t) => t) as tag (tag)}
        <li>
            <a
                href="/questions/tagged/{encodeURIComponent(tag)}"
                class="rounded bg-tag px-1.5 py-1 text-xs text-brand-dark hover:bg-blue-100"
                >{tag}</a
            >
        </li>
    {/each}
</ul>
