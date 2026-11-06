<script lang="ts">
    import type { Snippet } from 'svelte';

    let {
        open = $bindable(false),
        title,
        id,
        children
    }: { open?: boolean; title: string; id?: string; children: Snippet } = $props();

    function onKeydown(e: KeyboardEvent) {
        if (open && e.key === 'Escape') open = false;
    }
</script>

<svelte:window onkeydown={onKeydown} />

{#if open}
    <div
        class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4"
        role="presentation"
        onclick={(e) => {
            if (e.target === e.currentTarget) open = false;
        }}
    >
        <div
            {id}
            class="w-full max-w-lg rounded-lg bg-white shadow-xl"
            role="dialog"
            aria-modal="true"
            aria-label={title}
        >
            <div class="border-b border-gray-200 px-5 py-3">
                <h5 class="text-lg font-medium">{title}</h5>
            </div>
            <div class="px-5 py-4">
                {@render children()}
            </div>
        </div>
    </div>
{/if}
