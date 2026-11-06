<script lang="ts">
    import { goto } from '$app/navigation';
    import { onDestroy } from 'svelte';
    import * as api from '#lib/api.ts';
    import Editor from '#lib/components/Editor.svelte';
    import TagInput from '#lib/components/TagInput.svelte';
    import { firstDuplicateTag } from '#lib/tags.ts';
    import type { QuestionRow } from '#lib/types.ts';

    let title = $state('');
    let body = $state('');
    let tags = $state<string[]>([]);
    let formError = $state('');
    let submitting = $state(false);

    // "Similar questions": while the title is typed, search existing questions
    // so the user can find an answer before posting a duplicate.
    const SIMILAR_MIN_LEN = 3;
    const SIMILAR_LIMIT = 10;
    const SIMILAR_DEBOUNCE_MS = 300;
    let similar = $state<QuestionRow[]>([]);
    let similarOpen = $state(false);
    let similarLoading = $state(false);
    let similarRequest = 0;
    let debounce: ReturnType<typeof setTimeout> | undefined;

    async function searchSimilar() {
        const q = title.trim();
        if (q.length < SIMILAR_MIN_LEN) {
            similar = [];
            similarOpen = false;
            return;
        }
        const mine = ++similarRequest;
        similarLoading = true;
        try {
            const res = await api.post('search-questions/', { search: q, questions_per_page: SIMILAR_LIMIT });
            if (mine !== similarRequest) return;
            const j = await api.json<{ data?: { questions: QuestionRow[] } }>(res);
            similar = (res.ok && j.data?.questions) || [];
            similarOpen = similar.length > 0;
        } catch {
            if (mine === similarRequest) {
                similar = [];
                similarOpen = false;
            }
        } finally {
            if (mine === similarRequest) similarLoading = false;
        }
    }

    function onTitleInput() {
        clearTimeout(debounce);
        debounce = setTimeout(searchSimilar, SIMILAR_DEBOUNCE_MS);
    }

    onDestroy(() => clearTimeout(debounce));

    async function submit(e: SubmitEvent) {
        e.preventDefault();
        if (body.length < 20 || body.length > 100000) {
            formError = 'Question length should be 20 to 1000000 Characters!';
            return;
        }
        if (tags.length < 1) {
            formError = 'At least one tag should be supplied.';
            return;
        }
        if (tags.length > 5) {
            formError = 'Too many tags.';
            return;
        }
        const duplicate = firstDuplicateTag(tags);
        if (duplicate) {
            formError = `Duplicate tag: ${duplicate}`;
            return;
        }
        formError = '';
        submitting = true;
        const res = await api.post('create-question', { title, description: body, tag_list: tags });
        const j = await api.json<{ data?: { id: string; slug: string }; message?: string }>(res);
        submitting = false;
        if (res.status === 200 && j.data?.id) {
            await goto(`/questions/${j.data.id}/${j.data.slug}`);
        } else {
            formError = j.message || 'Something went wrong, try again!';
        }
    }
</script>

<svelte:head>
    <title>Ask a question ❤ Kunjika</title>
</svelte:head>

<div class="mt-5">
    {#if formError}<div class="alert-danger mb-4" role="alert" id="askError">{formError}</div>{/if}
    <h4 class="text-xl font-normal">Post your question for discussion</h4>
    <hr class="my-3 border-gray-200" />
    <form onsubmit={submit} class="space-y-4">
        <div class="relative">
            <label for="title" class="label">Summary</label>
            <input
                bind:value={title}
                oninput={onTitleInput}
                onblur={() => setTimeout(() => (similarOpen = false), 150)}
                class="input"
                id="title"
                type="text"
                minlength="6"
                maxlength="256"
                required
                role="combobox"
                aria-autocomplete="list"
                aria-expanded={similarOpen}
                aria-controls="similarQuestions"
                autocomplete="off"
            />
            <div
                id="similarQuestions"
                role="listbox"
                hidden={!similarOpen}
                class="absolute z-20 mt-1 w-full rounded-md border border-gray-200 bg-white py-1 text-sm shadow-lg"
            >
                {#if similarLoading}<div class="px-3 py-1.5 text-gray-500">Searching…</div>{/if}
                <div class="px-3 py-1 text-xs text-gray-500">Similar questions</div>
                {#each similar as q (q.id)}
                    <a role="option" aria-selected="false" class="block px-3 py-1.5 text-brand-dark hover:bg-gray-100" href="/questions/{q.id}/{q.slug}" target="_blank" rel="noopener"
                        >{q.title}</a
                    >
                {/each}
            </div>
        </div>
        <div>
            <span class="label">Question</span>
            <Editor bind:value={body} />
        </div>
        <div>
            <label for="tags" class="label">Tags</label>
            <TagInput bind:tags />
        </div>
        <div class="flex justify-center">
            <button type="submit" class="btn-primary" disabled={submitting}> Ask </button>
        </div>
    </form>
</div>
