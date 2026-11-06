<script lang="ts">
    import { onMount } from 'svelte';

    let p = $state(0);
    let visible = $state(false);

    onMount(() => {
        visible = true;
        let timer: ReturnType<typeof setTimeout>;
        function next() {
            p += 0.1;
            const remaining = 1 - p;
            if (remaining > 0.15) timer = setTimeout(next, 500 / remaining);
        }
        timer = setTimeout(next, 250);
        return () => clearTimeout(timer);
    });
</script>

{#if visible}
    <div class="fixed inset-x-0 top-0 z-50 h-1">
        <div class="h-full bg-brand transition-[width] duration-500" style="width: {p * 100}%"></div>
    </div>
{/if}
{#if p >= 0.4}
    <div class="pointer-events-none fixed inset-0 z-40 animate-pulse bg-white/30"></div>
{/if}
