<script lang="ts">
    import { browser } from '$app/env';
    import { page } from '$app/state';
    import * as api from '#lib/api.ts';
    import { isReviewer } from '#lib/types.ts';
    import Icon, { type IconName } from './Icon.svelte';

    let reviewer = $derived(isReviewer(page.data.user));
    let openCount = $state(0);

    $effect(() => {
        if (!browser || !reviewer) return;
        api.post('reports/open-count/', {})
            .then(async (res) => {
                if (res.status === 200) openCount = (await api.json<{ count?: number }>(res)).count ?? 0;
            })
            .catch(() => {}); // best-effort; the queue page is the source of truth
    });

    const links: { href: string; label: string; icon: IconName }[] = [
        { href: '/questions', label: 'Questions', icon: 'questions' },
        { href: '/tags', label: 'Tags', icon: 'tags' },
        { href: '/users', label: 'Users', icon: 'users' }
    ];

    const item =
        'flex items-center gap-3 rounded-lg px-3 py-2 text-sm text-gray-700 hover:bg-gray-100 hover:text-gray-900';
    const activeItem = 'bg-sky-50 font-semibold text-sky-900';
</script>

<aside class="w-full shrink-0 border-gray-200 px-3 py-4 md:sticky md:top-14 md:w-[232px] md:border-r">
    <nav aria-label="Primary">
        <ul class="flex gap-1 overflow-x-auto md:flex-col">
            {#each links as l (l.href)}
                <li>
                    <a href={l.href} class="{item} {page.url.pathname.startsWith(l.href) ? activeItem : ''}">
                        <Icon name={l.icon} /><span>{l.label}</span>
                    </a>
                </li>
            {/each}
            {#if reviewer}
                <li>
                    <a href="/reports" class="{item} {page.url.pathname.startsWith('/reports') ? activeItem : ''}">
                        <Icon name="flag" /><span>Reports</span>
                        {#if openCount > 0}
                            <span class="ml-auto rounded-full bg-danger px-2 text-xs text-white">{openCount}</span>
                        {/if}
                    </a>
                </li>
                <li>
                    <a
                        href="/moderation-log"
                        class="{item} {page.url.pathname.startsWith('/moderation-log') ? activeItem : ''}"
                    >
                        <Icon name="history" /><span>Moderation log</span>
                    </a>
                </li>
            {/if}
            <li class="my-2 hidden border-t border-gray-200 md:block" role="separator"></li>
            <li class="hidden md:block">
                <a href="https://github.com/Nalanda-Labs/kunjika" target="_blank" rel="noopener noreferrer" class={item}>
                    <Icon name="github" /><span>GitHub</span>
                </a>
            </li>
        </ul>
    </nav>
</aside>
