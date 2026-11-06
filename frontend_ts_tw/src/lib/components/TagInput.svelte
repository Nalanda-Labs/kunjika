<script lang="ts">
    import * as api from '#lib/api.ts';

    // Tab, Enter or "," commits the typed tag; Backspace/Escape on an empty
    // input removes the last one; pasting "a, b" adds both; leaving the field
    // commits what was typed. Only exact repeats are blocked here - the form
    // checks case-insensitive duplicates, since tag names are case-insensitive.
    let {
        tags = $bindable<string[]>([]),
        id = 'tags',
        placeholder = 'Tags, tab to complete',
        max = 5
    }: { tags?: string[]; id?: string; placeholder?: string; max?: number } = $props();

    let text = $state('');
    let suggestions = $state<string[]>([]);
    let active = $state(-1);
    let requestId = 0;

    function add(raw: string) {
        for (const part of raw.split(',')) {
            const t = part.trim();
            if (t && !tags.includes(t)) tags = [...tags, t];
        }
        text = '';
        closeSuggestions();
    }

    function closeSuggestions() {
        requestId++; // a request still in flight must not reopen the list
        suggestions = [];
        active = -1;
    }

    function remove(i: number) {
        tags = tags.filter((_, j) => j !== i);
    }

    async function suggest() {
        const q = text.trim();
        const mine = ++requestId;
        if (q.length < 1) {
            suggestions = [];
            return;
        }
        try {
            const res = await api.post('get-tags', { tag: q });
            if (mine !== requestId) return;
            const j = await api.json<{ data?: { name: string }[] }>(res);
            suggestions = (j.data ?? []).map((t) => t.name).filter((n) => !tags.includes(n));
            active = -1;
        } catch {
            suggestions = [];
        }
    }

    function onKeydown(e: KeyboardEvent) {
        if (e.key === 'ArrowDown' && suggestions.length) {
            e.preventDefault();
            active = (active + 1) % suggestions.length;
        } else if (e.key === 'ArrowUp' && suggestions.length) {
            e.preventDefault();
            active = (active - 1 + suggestions.length) % suggestions.length;
        } else if (e.key === 'Tab' || e.key === 'Enter' || e.key === ',') {
            const pick = active >= 0 ? suggestions[active] : text;
            if (pick.trim()) {
                e.preventDefault();
                add(pick);
            } else if (e.key === 'Enter') {
                e.preventDefault();
            }
        } else if ((e.key === 'Backspace' || e.key === 'Escape') && text === '' && tags.length) {
            remove(tags.length - 1);
        }
    }

    function onPaste(e: ClipboardEvent) {
        const pasted = e.clipboardData?.getData('text') ?? '';
        if (pasted.includes(',')) {
            e.preventDefault();
            add(pasted);
        }
    }
</script>

<div class="relative">
    <div
        class="flex min-h-10 flex-wrap items-center gap-1.5 rounded-md border border-gray-300 bg-white px-2 py-1.5 focus-within:border-brand focus-within:ring-2 focus-within:ring-brand/30"
    >
        {#each tags as tag, i (tag)}
            <span class="inline-flex items-center gap-1 rounded bg-tag px-2 py-0.5 text-xs text-brand-dark">
                {tag}
                <button
                    type="button"
                    class="text-gray-500 hover:text-gray-900"
                    aria-label="Remove {tag}"
                    onclick={() => remove(i)}>×</button
                >
            </span>
        {/each}
        <input
            {id}
            type="text"
            bind:value={text}
            oninput={suggest}
            onkeydown={onKeydown}
            onpaste={onPaste}
            onblur={() =>
                setTimeout(() => {
                    if (text.trim()) add(text);
                    else closeSuggestions();
                }, 150)}
            placeholder={tags.length >= max ? '' : placeholder}
            autocomplete="off"
            role="combobox"
            aria-expanded={suggestions.length > 0}
            aria-controls="{id}-suggestions"
            class="min-w-32 flex-1 border-0 p-0.5 text-sm focus:ring-0 focus:outline-none"
        />
    </div>
    {#if suggestions.length}
        <ul
            id="{id}-suggestions"
            role="listbox"
            class="absolute z-20 mt-1 max-h-60 w-full overflow-auto rounded-md border border-gray-200 bg-white py-1 text-sm shadow-lg"
        >
            {#each suggestions as s, i (s)}
                <li role="option" aria-selected={i === active}>
                    <button
                        type="button"
                        class="block w-full px-3 py-1.5 text-left hover:bg-gray-100 {i === active ? 'bg-gray-100' : ''}"
                        onmousedown={(e) => {
                            e.preventDefault();
                            add(s);
                        }}>{s}</button
                    >
                </li>
            {/each}
        </ul>
    {/if}
</div>
