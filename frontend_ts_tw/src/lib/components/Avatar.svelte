<script lang="ts">
    let {
        src,
        name,
        size = 48
    }: { src: string | null | undefined; name: string; size?: number } = $props();
</script>

{#if src}
    <img
        {src}
        alt="{name}'s avatar"
        width={size}
        height={size}
        class="rounded object-cover"
        style="width:{size}px;height:{size}px"
    />
{:else}
    <span
        class="inline-flex items-center justify-center rounded-full bg-[#0f9d58] font-medium text-white"
        style="width:{size}px;height:{size}px;font-size:{size / 2.5}px"
        aria-label={name}>{(name || '?')[0].toUpperCase()}</span
    >
{/if}
