<script lang="ts">
    import type { QuestionRow } from '#lib/types.ts';
    import { listAge } from '#lib/time.ts';
    import Icon from './Icon.svelte';
    import TagChips from './TagChips.svelte';

    let { q, pinned = false }: { q: QuestionRow; pinned?: boolean } = $props();
</script>

<article class="flex gap-3 border-b border-gray-200 py-4">
    <div
        class="flex h-14 w-16 shrink-0 flex-col items-center justify-center rounded {q.answer_accepted
            ? 'bg-green-600 text-white'
            : 'text-gray-700'}"
    >
        <span class="text-base">{q.answers}</span>
        <span class="text-[10px]">answers</span>
    </div>
    <div class="flex h-14 w-16 shrink-0 flex-col items-center justify-center text-gray-700">
        <span class="text-base">{q.views}</span>
        <span class="text-[10px]">views</span>
    </div>
    <div class="min-w-0 flex-1">
        <a href="/questions/{q.id}/{q.slug}" class="text-base font-normal text-brand-dark hover:underline">
            {#if pinned}<Icon name="pin" size={16} class="mr-1 align-[-2px]" />{/if}{q.title}
        </a>
        <div class="mt-3 flex flex-wrap items-center justify-between gap-2">
            <TagChips tags={q.tags.split(',')} />
            <span class="text-xs text-gray-500"
                >{listAge(q.cat, q.uat)}
                <a href="/users/{q.uid}/{q.username}" class="text-brand">{q.username}</a></span
            >
        </div>
    </div>
</article>
