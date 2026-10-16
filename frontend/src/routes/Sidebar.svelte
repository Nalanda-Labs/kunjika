<script>
    import { page } from '$app/stores';
    import * as api from '$lib/api.js';
    import { browser } from '$app/environment';
    import getCookie from '../lib/cookie.js';

    $: isReviewer =
        $page.data.user &&
        ($page.data.user.is_moderator || $page.data.user.is_superuser);

    let openCount = 0;

    async function loadOpenCount() {
        if (!browser || !isReviewer) {
            return;
        }
        try {
            const xsrf_token = getCookie('xsrf_token');
            const response = await api.post('reports/open-count/', {}, xsrf_token);
            if (response.status === 200) {
                const j = JSON.parse(await response.text());
                openCount = j.count || 0;
            }
        } catch (e) {
            // badge is best-effort; the queue page is the source of truth
        }
    }

    $: if (browser && isReviewer) {
        loadOpenCount();
    }
</script>

<aside class="sidebar">
    <nav aria-label="Primary">
        <ul>
            <li>
                <a
                    rel="prefetch"
                    href="/questions"
                    class:active={$page.url.pathname.startsWith('/questions')}
                >
                    <i class="fa-solid fa-comment-dots" aria-hidden="true"></i>
                    <span>Questions</span>
                </a>
            </li>
            <li>
                <a
                    rel="prefetch"
                    href="/tags"
                    class:active={$page.url.pathname.startsWith('/tags')}
                >
                    <i class="fa-solid fa-tags" aria-hidden="true"></i>
                    <span>Tags</span>
                </a>
            </li>
            <li>
                <a
                    rel="prefetch"
                    href="/users"
                    class:active={$page.url.pathname.startsWith('/users')}
                >
                    <i class="fa-solid fa-users" aria-hidden="true"></i>
                    <span>Users</span>
                </a>
            </li>
            {#if isReviewer}
                <li>
                    <a
                        rel="prefetch"
                        href="/reports"
                        class:active={$page.url.pathname.startsWith('/reports')}
                    >
                        <i class="fa-solid fa-flag" aria-hidden="true"></i>
                        <span>Reports</span>
                        {#if openCount > 0}
                            <span class="report-badge">{openCount}</span>
                        {/if}
                    </a>
                </li>
            {/if}
            <li class="sidebar-divider" role="separator"></li>
            <li>
                <a
                    href="https://github.com/Nalanda-Labs/kunjika"
                    target="_blank"
                    rel="noopener noreferrer"
                    aria-label="github"
                >
                    <i class="fa-brands fa-github" aria-hidden="true"></i>
                    <span>GitHub</span>
                </a>
            </li>
        </ul>
    </nav>
</aside>

<style>
    .sidebar {
        flex: 0 0 var(--sidebar-width, 232px);
        width: var(--sidebar-width, 232px);
        align-self: stretch;
        background: #ffffff;
        border-right: 1px solid #e6e6e6;
        padding: 1.25rem 0.85rem;
    }

    .sidebar ul {
        list-style: none;
        margin: 0;
        padding: 0;
        display: flex;
        flex-direction: column;
        gap: 0.15rem;
    }

    .sidebar a {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        padding: 0.6rem 0.85rem;
        border-radius: 6px;
        color: #455a64;
        text-decoration: none;
        font-weight: 400;
        transition:
            background 0.15s,
            color 0.15s;
    }

    .sidebar a i {
        width: 20px;
        text-align: center;
        font-size: 1rem;
        color: #78909c;
        transition: color 0.15s;
    }

    .sidebar a:hover {
        background: #f1f5f9;
        color: #0277bd;
    }

    .sidebar a:hover i {
        color: #0288d1;
    }

    .sidebar a.active {
        background: #e1f5fe;
        color: #01579b;
        font-weight: 600;
    }

    .sidebar a.active i {
        color: #0288d1;
    }

    .sidebar-divider {
        height: 1px;
        background: #eceff1;
        margin: 0.6rem 0.5rem;
    }

    .report-badge {
        margin-left: auto;
        background: #d32f2f;
        color: #fff;
        font-size: 0.75rem;
        font-weight: 600;
        border-radius: 999px;
        padding: 0 0.5rem;
        line-height: 1.4rem;
    }

    @media (max-width: 767.98px) {
        .sidebar {
            position: static;
            height: auto;
            width: 100%;
            flex: 0 0 auto;
            border-right: none;
            border-bottom: 1px solid #e6e6e6;
            padding: 0.5rem;
        }

        .sidebar ul {
            flex-direction: row;
            flex-wrap: wrap;
            gap: 0.25rem;
        }

        .sidebar a {
            padding: 0.4rem 0.7rem;
        }
    }
</style>
