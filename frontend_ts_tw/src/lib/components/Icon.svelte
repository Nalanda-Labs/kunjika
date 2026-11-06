<script lang="ts" module>
    // Outline icons (24×24, stroked) in the style of Lucide; `github` is filled.
    const PATHS = {
        questions: ['M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z'],
        tags: [
            'M12.59 2.59A2 2 0 0 0 11.17 2H4a2 2 0 0 0-2 2v7.17a2 2 0 0 0 .59 1.41l8.7 8.71a2.43 2.43 0 0 0 3.42 0l6.58-6.58a2.43 2.43 0 0 0 0-3.42z',
            'M7.5 7.5h.01'
        ],
        users: [
            'M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2',
            'M13 7a4 4 0 1 1-8 0 4 4 0 0 1 8 0',
            'M22 21v-2a4 4 0 0 0-3-3.87',
            'M16 3.13a4 4 0 0 1 0 7.75'
        ],
        flag: ['M4 15s1-1 4-1 5 2 8 2 4-1 4-1V3s-1 1-4 1-5-2-8-2-4 1-4 1z', 'M4 22v-7'],
        search: ['M21 21l-4.3-4.3', 'M19 11a8 8 0 1 1-16 0 8 8 0 0 1 16 0'],
        bold: ['M6 12h9a4 4 0 0 1 0 8H7a1 1 0 0 1-1-1V5a1 1 0 0 1 1-1h7a4 4 0 0 1 0 8'],
        italic: ['M19 4h-9', 'M14 20H5', 'M15 4 9 20'],
        link: [
            'M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71',
            'M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71'
        ],
        list: ['M8 6h13', 'M8 12h13', 'M8 18h13', 'M3 6h.01', 'M3 12h.01', 'M3 18h.01'],
        'list-ordered': [
            'M10 6h11',
            'M10 12h11',
            'M10 18h11',
            'M4 6h1v4',
            'M4 10h2',
            'M6 18H4c0-1 2-2 2-3s-1-1.5-2-1'
        ],
        image: [
            'M5 3h14a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2z',
            'M10 9a1 1 0 1 1-2 0 1 1 0 0 1 2 0',
            'M21 15l-3.09-3.09a2 2 0 0 0-2.82 0L6 21'
        ],
        code: ['M16 18l6-6-6-6', 'M8 6l-6 6 6 6'],
        note: [
            'M15.5 3H5a2 2 0 0 0-2 2v14c0 1.1.9 2 2 2h14a2 2 0 0 0 2-2V8.5L15.5 3Z',
            'M15 3v6h6'
        ],
        info: ['M22 12a10 10 0 1 1-20 0 10 10 0 0 1 20 0', 'M12 16v-4', 'M12 8h.01'],
        warning: [
            'M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z',
            'M12 9v4',
            'M12 17h.01'
        ],
        help: [
            'M22 12a10 10 0 1 1-20 0 10 10 0 0 1 20 0',
            'M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3',
            'M12 17h.01'
        ],
        'chevron-up': ['M18 15l-6-6-6 6'],
        'chevron-down': ['M6 9l6 6 6-6'],
        'chevron-left': ['M15 18l-6-6 6-6'],
        'chevron-right': ['M9 18l6-6-6-6'],
        'chevrons-left': ['M11 17l-5-5 5-5', 'M18 17l-5-5 5-5'],
        check: ['M20 6 9 17l-5-5'],
        edit: ['M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z'],
        pin: [
            'M12 17v5',
            'M9 10.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24V16a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1v-.76a2 2 0 0 0-1.11-1.79l-1.78-.9A2 2 0 0 1 15 10.76V7a1 1 0 0 1 1-1 2 2 0 0 0 0-4H8a2 2 0 0 0 0 4 1 1 0 0 1 1 1z'
        ],
        bookmark: ['M19 21l-7-4-7 4V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2v16z'],
        reply: ['M9 17l-5-5 5-5', 'M20 18v-2a4 4 0 0 0-4-4H4'],
        share: ['M4 12v8a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-8', 'M16 6l-4-4-4 4', 'M12 2v13'],
        menu: ['M4 6h16', 'M4 12h16', 'M4 18h16'],
        x: ['M18 6 6 18', 'M6 6l12 12']
    } as const;

    const GITHUB =
        'M12 .3a12 12 0 0 0-3.8 23.4c.6.1.8-.3.8-.6v-2c-3.3.7-4-1.6-4-1.6-.6-1.4-1.4-1.8-1.4-1.8-1-.7.1-.7.1-.7 1.2 0 1.9 1.2 1.9 1.2 1 1.8 2.8 1.3 3.5 1 0-.8.4-1.3.7-1.6-2.7-.3-5.5-1.3-5.5-6 0-1.2.5-2.3 1.3-3.1-.2-.4-.6-1.6 0-3.2 0 0 1-.3 3.4 1.2a11.5 11.5 0 0 1 6 0c2.3-1.5 3.3-1.2 3.3-1.2.6 1.6.2 2.8 0 3.2.9.8 1.3 1.9 1.3 3.2 0 4.6-2.8 5.6-5.5 5.9.5.4.9 1.1.9 2.3v3.3c0 .3.1.7.8.6A12 12 0 0 0 12 .3';

    export type IconName = keyof typeof PATHS | 'github';
</script>

<script lang="ts">
    let {
        name,
        size = 18,
        class: klass = ''
    }: { name: IconName; size?: number; class?: string } = $props();
</script>

{#if name === 'github'}
    <svg
        width={size}
        height={size}
        viewBox="0 0 24 24"
        fill="currentColor"
        aria-hidden="true"
        class="inline-block shrink-0 {klass}"><path d={GITHUB} /></svg
    >
{:else}
    <svg
        width={size}
        height={size}
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
        aria-hidden="true"
        class="inline-block shrink-0 {klass}"
    >
        {#each PATHS[name] as d (d)}<path {d} />{/each}
    </svg>
{/if}
