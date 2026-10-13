<script>
    import { page } from '$app/stores';
    import { goto } from '$app/navigation';

    let search = '';

    function submitSearch(event) {
        // Prevent a full-page navigation so SvelteKit can handle the route.
        event.preventDefault();
        const q = search.trim();
        if (q === '') {
            return;
        }
        goto(`/search?q=${encodeURIComponent(q)}`);
        // Collapse the navbar on mobile after searching.
        const el = document.getElementById('navbarSupportedContent');
        if (el) {
            el.classList.remove('show');
        }
    }
</script>

<nav class="navbar navbar-expand-lg bg-body-tertiary">
    <div class="container">
        <a class="navbar-brand" href="/">Kunjika</a>
        <button
            class="navbar-toggler"
            type="button"
            data-bs-toggle="collapse"
            data-bs-target="#navbarSupportedContent"
            aria-controls="navbarSupportedContent"
            aria-expanded="false"
            aria-label="Toggle navigation"
        >
            <span class="navbar-toggler-icon"></span>
        </button>
        <div class="collapse navbar-collapse" id="navbarSupportedContent">
            <form
                class="d-flex me-auto"
                role="search"
                method="GET"
                action="/search"
                on:submit={submitSearch}
            >
                <input
                    class="form-control search-input"
                    type="search"
                    name="q"
                    placeholder="Search questions"
                    aria-label="Search questions"
                    bind:value={search}
                />
            </form>
            <ul class="navbar-nav ms-auto mb-2 mb-lg-0">
                {#if $page.data.user}
                    <li class="nav-item">
                        <a
                            rel="prefetch"
                            class="nav-link"
                            href="/users/{$page.data.user.id}/{$page.data.user.username}"
                        >
                            <img
                                src={$page.data.user.image_url}
                                alt="profile"
                                style="width:32px;height:32px"
                            />
                        </a>
                    </li>
                    <li class="nav-item">
                        <form style="margin-top:5px;" method="POST" action="/logout">
                            <button class="btn btn-primary" style="color:#fff" type="submit"
                                >Sign Out</button
                            >
                        </form>
                    </li>
                {:else}
                    <li class="nav-item">
                        <a rel="prefetch" href="/login" class="nav-link"> Login </a>
                    </li>

                    <li class="nav-item">
                        <a rel="prefetch" href="/register" class="nav-link"> Register </a>
                    </li>
                {/if}
            </ul>
        </div>
    </div>
</nav>

<style>
    /* Keep the navbar bar full-width, but shift its inner contents to the
       right by the sidebar width so the brand + auth controls line up with
       the page body (which sits to the right of the left sidebar). */
    .navbar {
        padding-left: var(--sidebar-width, 232px);
    }

    .container {
        max-width: 900px !important;
    }

    .search-input {
        min-width: 600px;
        border-radius: 25px;
    }

    @media (max-width: 767.98px) {
        /* On small screens the sidebar becomes a full-width row above the
           content, so the navbar no longer needs the left offset. */
        .navbar {
            padding-left: 0;
        }
    }
</style>
