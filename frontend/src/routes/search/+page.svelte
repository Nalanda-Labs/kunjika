<script context="module">
    export function preload({ params }, { user }) {}
</script>

<script>
    import { page } from '$app/stores';
    import * as api from '$lib/api.js';

    let questions = [];
    let count = 0;
    // Request the top 50 most relevant matches; the backend clamps the response
    // to this limit too (`SEARCH_RESULTS_LIMIT`).
    let questions_per_page = 50;
    let loading = false;

    $: q = $page.url.searchParams.get('q') || '';

    function processQuestions(data) {
        questions = data;
        for (let i = 0; i < questions.length; i++) {
            questions[i]['tags'] = questions[i]['tags'].split(',');
            questions[i]['tid'] = questions[i]['tid'].split(',');
            let asked_ts = new Date(questions[i].cat * 1000);
            let updated_ts = new Date(questions[i].uat * 1000);
            let now = new Date();
            let shown_ts;

            if (asked_ts.getTime() == updated_ts.getTime()) {
                shown_ts = Math.floor((now - asked_ts) / 1000);

                if (shown_ts >= 259200) {
                    let year = asked_ts.getYear() + 1900;
                    let month = asked_ts.getMonth() + 1;
                    shown_ts = 'asked on ' + asked_ts.getDate() + '/' + month + '/' + year;
                } else if (172800 <= shown_ts && shown_ts < 259200) {
                    shown_ts = 'asked 2 days ago';
                } else if (86400 <= shown_ts && shown_ts < 172800) {
                    shown_ts = 'asked yesterday';
                } else if (3600 <= shown_ts && shown_ts < 86400) {
                    shown_ts = 'asked ' + Math.floor(shown_ts / 3600) + 'h ago';
                } else if (60 <= shown_ts && shown_ts < 3600) {
                    shown_ts = 'asked ' + Math.floor(shown_ts / 60) + 'm ago';
                } else {
                    shown_ts = 'asked ' + shown_ts + 's ago';
                }
            } else {
                shown_ts = Math.floor((now - updated_ts) / 1000);

                if (shown_ts >= 259200) {
                    let year = updated_ts.getYear() + 1900;
                    let month = updated_ts.getMonth() + 1;
                    shown_ts = 'modified on ' + updated_ts.getDate() + '/' + month + '/' + year;
                } else if (172800 <= shown_ts && shown_ts < 259200) {
                    shown_ts = 'modified 2 days ago';
                } else if (86400 <= shown_ts && shown_ts < 172800) {
                    shown_ts = 'modified yesterday';
                } else if (3600 <= shown_ts && shown_ts < 86400) {
                    shown_ts = 'modified ' + Math.floor(shown_ts / 3600) + 'h ago';
                } else if (60 <= shown_ts && shown_ts < 3600) {
                    shown_ts = 'modified ' + Math.floor(shown_ts / 60) + 'm ago';
                } else {
                    shown_ts = 'modified ' + shown_ts + 's ago';
                }
            }

            questions[i].shown_ts = shown_ts;
        }
    }

    async function runSearch(term) {
        const query = (term || '').trim();
        if (query === '') {
            questions = [];
            count = 0;
            return;
        }

        loading = true;
        try {
            let response = await api.post('search-questions/', {
                search: query,
                questions_per_page
            });

            if (response.status === 200) {
                response = JSON.parse(await response.text());

                if (response.hasOwnProperty('data')) {
                    let data = response.data.questions.map((t) => t);
                    count = response.count;
                    processQuestions(data);
                    // `processQuestions` mutates the array in place; reassign so
                    // Svelte re-renders with the computed `shown_ts`.
                    questions = questions;
                } else {
                    questions = [];
                    count = 0;
                }
            } else {
                questions = [];
                count = 0;
            }
        } catch (e) {
            questions = [];
            count = 0;
        } finally {
            loading = false;
        }
    }

    // Run on first load and whenever the `q` query parameter changes (e.g. a new
    // search submitted from the navbar).
    $: runSearch(q);
</script>

<svelte:head>
    <title>Search{q ? `: ${q}` : ''} ❤ Kunjika</title>
</svelte:head>
<div style="margin-top:20px">
    <h4 class="text-xl font-bold">Search results for "{q}" ({count})</h4>
    {#if loading}
        <p>Searching…</p>
    {/if}
    {#if !loading && q.trim() !== '' && count === 0}
        <p>No questions matched your search.</p>
    {/if}
    <div class="row">
        {#each questions as { id, slug, title, tags, shown_ts, uid, username, answers, views, answer_accepted }}
            <hr
                style="border-bottom:1px solid;color:#ccc;display:block;min-width:100%;margin-top:20px;margin-bottom:20px"
            />
            {#if answer_accepted}
                <div
                    style="margin-right:0px;flex-basis: 60px;min-width:60px;max-width:60px;height:60px;float:left;background-color: green;color: white;"
                >
                    <p style="text-align:center;font-size:16px;margin-top:5px">
                        {answers}
                    </p>
                    <p style="text-align:center;font-size:10px;margin-top:0px;">
                        answers
                    </p>
                </div>
            {:else}
                <div
                    style="margin-right:0px;flex-basis: 60px;min-width:60px;max-width:60px;height:60px;float:left;"
                >
                    <p style="text-align:center;font-size:16px;margin-top:5px">
                        {answers}
                    </p>
                    <p style="text-align:center;font-size:10px;margin-top:0px;">
                        answers
                    </p>
                </div>
            {/if}
            <div
                style="margin-right:0px;flex-basis: 60px;min-width:60px;max-width:60px;height:60px;float:left"
            >
                <p style="text-align:center;font-size:16px;margin-top:5px">
                    {views}
                </p>
                <p style="text-align:center;font-size:10px;margin-top:10px;">
                    views
                </p>
            </div>
            <div style="width:85%;float:left;position:relative">
                <a
                    href="/questions/{id}/{slug}"
                    class="blue-text text-darken-2"
                    style="text-decoration:none; font-size:16px; font-weight:400"
                    >{title}</a
                >
                <div style="margin-top:20px;clear:both"></div>
                {#each tags as tag, i}
                    <a
                        href="/questions/tagged/{encodeURIComponent(tag)}"
                        class="light-blue darken-2"
                        style="display:inline;padding:5px;border-radius:3px;text-decoration:none; background-color:#f0f0ff;margin-right:10px;font-size:12px"
                        >{tag}</a
                    >
                {/each}
                <span style="float:right"
                    >{shown_ts}
                    <a
                        href="/users/{uid}/{username}"
                        style="text-decoration:none; color:#4285F4;">{username}</a
                    ></span
                >
            </div>
            <div style="clear:both"></div>
        {/each}
    </div>
</div>

