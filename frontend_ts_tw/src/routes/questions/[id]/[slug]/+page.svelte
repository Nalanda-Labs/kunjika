<script lang="ts">
    import { goto } from '$app/navigation';
    import { page } from '$app/state';
    import { onMount, tick } from 'svelte';
    import * as api from '#lib/api.ts';
    import Avatar from '#lib/components/Avatar.svelte';
    import Editor from '#lib/components/Editor.svelte';
    import Icon from '#lib/components/Icon.svelte';
    import Markdown from '#lib/components/Markdown.svelte';
    import ReportDialog from '#lib/components/ReportDialog.svelte';
    import TagChips from '#lib/components/TagChips.svelte';
    import { loadMathJax, parseMarkdown } from '#lib/markdown.ts';
    import { cursor, timeAgo } from '#lib/time.ts';
    import { isReviewer, type AnswerRow, type QuestionDetail } from '#lib/types.ts';

    const ANSWERS_PER_LOAD = 50;

    interface Answer extends AnswerRow {
        html: string;
        myVote: number;
    }

    let id = $derived(page.params.id ?? '');
    let slug = $derived(page.params.slug ?? '');
    let user = $derived(page.data.user);

    let question = $state<QuestionDetail | null>(null);
    let questionHtml = $state('');
    let myVote = $state(0);
    let answers = $state<Answer[]>([]);
    let moreAnswers = $state(false);

    let editorOpen = $state(false);
    let replyTo = $state<{ id: string | number; username: string } | null>(null);
    let replyText = $state('');
    let replyError = $state('');
    let replying = $state(false);
    let editorEl = $state<HTMLElement>();
    let reportDialog = $state<ReturnType<typeof ReportDialog>>();

    let isAsker = $derived(!!user && !!question && String(user.id) === String(question.posted_by_id));

    function toAnswer(a: AnswerRow): Answer {
        return { ...a, html: parseMarkdown(a.description), myVote: a.vote_by_current_user };
    }

    async function loadAnswers(after: number) {
        const res = await api.post(`question/get-answers/${id}/?limit=${ANSWERS_PER_LOAD}`, {
            cat: cursor(after)
        });
        const j = await api.json<{ data?: { questions: AnswerRow[] } }>(res);
        const rows = j.data?.questions ?? [];
        // The cursor has one-second precision, so a page can repeat the last
        // answer of the previous one; skip anything already shown.
        const seen = new Set(answers.map((a) => a.question_id));
        answers = [...answers, ...rows.filter((r) => !seen.has(r.question_id)).map(toAnswer)];
        moreAnswers = rows.length === ANSWERS_PER_LOAD;
    }

    onMount(async () => {
        loadMathJax();
        const res = await api.get(`questions/${id}/${slug}`);
        if (res.status === 404) {
            goto('/404');
            return;
        }
        const j = await api.json<{ data?: QuestionDetail; message?: string }>(res);
        if (res.status !== 200 || !j.data) {
            alert(j.message || 'Could not load the question.');
            return;
        }
        question = j.data;
        questionHtml = parseMarkdown(j.data.description);
        myVote = j.data.vote_by_current_user;
        replyTo = { id: j.data.posted_by_id, username: j.data.username };

        await loadAnswers(j.data.cat);

        // Jump to and highlight an answer linked as /questions/{id}/{slug}#{answer_id}.
        await tick();
        const target = page.url.hash.slice(1);
        const el = target && document.getElementById(target);
        if (el) {
            el.scrollIntoView({ block: 'center' });
            el.classList.add('bg-green-100', 'transition-colors');
        }
    });

    async function vote(value: 1 | -1, postId: string) {
        const res = await api.post('votes', { vote: value, id: Number(postId) });
        const j = await api.json<{ success?: boolean; count_by_user?: number; data?: string }>(res);
        if (res.status !== 200 || !j.success || j.count_by_user === undefined) {
            alert(j.data || 'Could not vote, try again.');
            return;
        }
        if (question && postId === question.id) {
            question.votes += value;
            myVote = j.count_by_user;
        } else {
            const a = answers.find((x) => x.question_id === postId);
            if (a) {
                a.votes += value;
                a.myVote = j.count_by_user;
            }
        }
    }

    async function accept(answerId: string) {
        const res = await api.post(`accept-answer/${id}/${answerId}/`, {});
        const j = await api.json<{ accepted?: boolean; message?: string }>(res);
        if (res.status !== 200) {
            alert(j.message || 'Something went wrong, try again!');
            return;
        }
        // The API says whether the clicked answer is now accepted (it toggles).
        for (const a of answers) a.answer_accepted = a.question_id === answerId && j.accepted !== false;
    }

    async function bookmark(answerId: string | number) {
        const res = await api.post(`bookmark/${id}/${answerId}`, {});
        const j = await api.json<{ message?: string }>(res);
        alert(res.status === 200 ? 'Bookmark updated.' : j.message || 'Could not update the bookmark.');
    }

    async function pin() {
        const res = await api.post(`bookmark/${id}`, {});
        const j = await api.json<{ message?: string }>(res);
        alert(j.message || 'Something went wrong, try again!');
    }

    async function share(answerId: string) {
        const url = `${location.origin}/questions/${id}/${slug}#${answerId}`;
        try {
            await navigator.clipboard.writeText(url);
            alert('Link copied to the clipboard.');
        } catch {
            prompt('Copy this link:', url);
        }
    }

    async function showEditor(to: { id: string | number; username: string }) {
        replyTo = to;
        editorOpen = true;
        await tick();
        editorEl?.scrollIntoView({ block: 'center' });
    }

    async function reply(e: SubmitEvent) {
        e.preventDefault();
        if (!user) {
            alert('You are not logged in.');
            return;
        }
        if (replyText.length < 20 || replyText.length > 100000) {
            replyError = 'Body should not be less than 20 or more than 100000 characters.';
            return;
        }
        replyError = '';
        replying = true;
        const res = await api.post('answer', { id, value: replyText, reply_to: replyTo?.id });
        const j = await api.json<{ data?: string; message?: string }>(res);
        replying = false;
        if (res.status !== 200 || !j.data) {
            replyError = j.message || 'Could not post your answer, try again.';
            return;
        }
        answers.push(
            toAnswer({
                question_id: j.data,
                description: replyText,
                votes: 0,
                posted_by_id: String(user.id),
                username: user.username,
                image_url: user.image_url,
                answer_accepted: false,
                reply_to_id: null,
                rusername: null,
                rimage_url: null,
                vote_by_current_user: 0,
                created_at: '',
                cat: Math.floor(Date.now() / 1000)
            })
        );
        replyText = '';
        editorOpen = false;
    }
</script>

<svelte:head>
    <title>{question?.title ?? ''}</title>
</svelte:head>

{#snippet votes(postId: string, ownerId: string, count: number, mine: number, what: string)}
    <div class="flex flex-col items-center text-2xl leading-none">
        {#if user && String(user.id) !== String(ownerId)}
            <a
                href="/vote-up"
                id="vu-{postId}"
                class={mine === 1 ? 'vote' : 'text-brand'}
                title="This {what} shows effort and research."
                onclick={(e) => {
                    e.preventDefault();
                    vote(1, postId);
                }}><Icon name="chevron-up" size={26} /></a
            >
        {/if}
        <span class="text-lg">{count}</span>
        {#if user && String(user.id) !== String(ownerId)}
            <a
                href="/vote-down"
                id="vd-{postId}"
                class={mine === -1 ? 'vote' : 'text-brand'}
                title="This {what} does not show effort and research."
                onclick={(e) => {
                    e.preventDefault();
                    vote(-1, postId);
                }}><Icon name="chevron-down" size={26} /></a
            >
        {/if}
    </div>
{/snippet}

{#if question}
    <div class="mt-5">
        <h3 class="text-2xl font-normal">{question.title}</h3>
        <hr class="my-3 border-gray-200" />

        <div class="flex gap-4">
            <div class="flex w-14 shrink-0 flex-col items-center gap-2">
                <a href="/users/{question.posted_by_id}/{question.username}">
                    <Avatar src={question.image_url} name={question.username} />
                </a>
                {@render votes(question.id, question.posted_by_id, question.votes, myVote, 'question')}
            </div>
            <div class="min-w-0 flex-1">
                <div class="flex justify-between text-sm">
                    <span class="font-bold text-gray-500">{question.username}</span>
                    <span class="text-gray-500">{timeAgo(question.cat, 'asked')}</span>
                </div>
                <div class="mt-3 min-h-36"><Markdown html={questionHtml} math /></div>
                <TagChips tags={question.tags} />
                {#if user}
                    <div class="mt-3 flex flex-wrap justify-end gap-4">
                        {#if isReviewer(user)}
                            <a
                                href="/questions/pin/{id}"
                                class="action-link"
                                title="Pin this post"
                                onclick={(e) => {
                                    e.preventDefault();
                                    pin();
                                }}><Icon name="pin" size={16} />Pin</a
                            >
                        {/if}
                        <a href="/questions/edit/{id}" class="action-link" title="Edit this post"
                            ><Icon name="edit" size={16} />Edit</a
                        >
                        <a
                            href="/report/{id}"
                            class="action-link text-danger"
                            title="Report abusive or inappropriate content"
                            onclick={(e) => {
                                e.preventDefault();
                                reportDialog?.open(id, question?.title ?? '');
                            }}><Icon name="flag" size={16} />Report</a
                        >
                        <a
                            href="/bookmark/{id}/0"
                            class="action-link"
                            title="Bookmark this post"
                            onclick={(e) => {
                                e.preventDefault();
                                bookmark(0);
                            }}><Icon name="bookmark" size={16} />Bookmark</a
                        >
                        <a
                            href="/reply"
                            class="action-link"
                            title="Reply to this post"
                            onclick={(e) => {
                                e.preventDefault();
                                showEditor({ id: question!.posted_by_id, username: question!.username });
                            }}><Icon name="reply" size={16} />Reply</a
                        >
                    </div>
                {/if}
            </div>
        </div>

        {#each answers as a (a.question_id)}
            <hr class="my-4 border-gray-200" />
            <div class="flex gap-4 rounded" id={a.question_id}>
                <div class="flex w-14 shrink-0 flex-col items-center gap-2">
                    <a href="/users/{a.posted_by_id}/{a.username}" title={a.username}>
                        <Avatar src={a.image_url} name={a.username} />
                    </a>
                    {@render votes(a.question_id, a.posted_by_id, a.votes, a.myVote, 'answer')}
                    {#if isAsker}
                        <a
                            href="/accept-answer"
                            id="a-{a.question_id}"
                            class="text-2xl {a.answer_accepted ? 'answer' : 'text-gray-400 hover:text-accept'}"
                            title={a.answer_accepted ? 'Accepted answer!' : 'Accept answer!'}
                            onclick={(e) => {
                                e.preventDefault();
                                accept(a.question_id);
                            }}><Icon name="check" size={26} /></a
                        >
                    {:else if a.answer_accepted}
                        <span class="answer" title="Accepted answer"><Icon name="check" size={26} /></span>
                    {/if}
                </div>
                <div class="min-w-0 flex-1">
                    <div class="flex flex-wrap items-center justify-between gap-2 text-sm">
                        <span class="font-bold text-gray-500">{a.username}</span>
                        <span class="flex items-center gap-2 text-gray-500">
                            {#if a.reply_to_id}
                                reply to
                                <a href="/users/{a.reply_to_id}/{a.rusername}" title={a.rusername ?? ''}>
                                    <Avatar src={a.rimage_url ? `${a.rimage_url}?s=32` : ''} name={a.rusername ?? '?'} size={24} />
                                </a>
                            {/if}
                            posted {timeAgo(a.cat)}
                        </span>
                    </div>
                    <div class="mt-3 min-h-24"><Markdown html={a.html} math /></div>
                    {#if user}
                        <div class="mt-3 flex flex-wrap justify-end gap-4">
                            <a href="/answers/edit/{a.question_id}" class="action-link" title="Edit your post"
                                ><Icon name="edit" size={16} />Edit</a
                            >
                            <a
                                href="/report/{a.question_id}"
                                class="action-link text-danger"
                                title="Report abusive or inappropriate content"
                                onclick={(e) => {
                                    e.preventDefault();
                                    reportDialog?.open(a.question_id, '');
                                }}><Icon name="flag" size={16} />Report</a
                            >
                            <a
                                href="/reply"
                                class="action-link"
                                title="Reply to this post"
                                onclick={(e) => {
                                    e.preventDefault();
                                    showEditor({ id: a.posted_by_id, username: a.username });
                                }}><Icon name="reply" size={16} />Reply</a
                            >
                            <a
                                href="/share/{a.question_id}"
                                class="action-link"
                                title="Share a link to this post"
                                onclick={(e) => {
                                    e.preventDefault();
                                    share(a.question_id);
                                }}><Icon name="share" size={16} />Share</a
                            >
                            <a
                                href="/bookmark/{id}/{a.question_id}"
                                class="action-link"
                                title="Bookmark this post"
                                onclick={(e) => {
                                    e.preventDefault();
                                    bookmark(a.question_id);
                                }}><Icon name="bookmark" size={16} />Bookmark</a
                            >
                        </div>
                    {/if}
                </div>
            </div>
        {/each}

        {#if moreAnswers}
            <div class="mt-4 flex justify-center">
                <button
                    type="button"
                    class="btn-secondary"
                    onclick={() => loadAnswers(answers[answers.length - 1].cat)}>Load more answers</button
                >
            </div>
        {/if}

        <hr class="my-4 border-gray-200" />
        {#if user}
            <a
                href="/reply"
                class="action-link"
                title="Reply to this post"
                onclick={(e) => {
                    e.preventDefault();
                    showEditor({ id: question!.posted_by_id, username: question!.username });
                }}><Icon name="reply" size={16} />Reply</a
            >
        {/if}

        <form onsubmit={reply} class="mt-3 space-y-3">
            {#if replyError}<div class="alert-danger" role="alert" id="replyError">{replyError}</div>{/if}
            {#if editorOpen}
                <div id="editor" bind:this={editorEl} class="space-y-3">
                    <span class="text-sm text-brand">Replying to @{replyTo?.username}</span>
                    <Editor bind:value={replyText} minlength={20} maxlength={100000} />
                    <div class="flex justify-center">
                        <button class="btn-primary" disabled={replying}>Reply</button>
                    </div>
                </div>
            {/if}
        </form>
    </div>
{/if}

<ReportDialog bind:this={reportDialog} />
