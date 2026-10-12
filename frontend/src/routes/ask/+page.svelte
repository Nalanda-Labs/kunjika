<script>
    import { goto } from '$app/navigation';
    import * as api from '$lib/api.js';
    import { page } from '$app/stores';
    import { browser } from '$app/environment';
    import { onDestroy } from 'svelte';
    import Tags from 'svelte-tags-input';
    import '../highlight.css';
    import { redirect } from '@sveltejs/kit';
    import Editor from '../../components/Editor/Editor.svelte';
    import Preview from '../../components/Editor/Preview.svelte';
    import getCookie from '../../lib/cookie.js';
    import '../../editor.css';
    import onImageUpload from '../../lib/imageUpload';
    import { firstDuplicateTag } from '../../lib/utils/tags.js';

    let title = '';
    let tagList = [];
    let contentValue = '',
        markup = '';
    let formError = '';

    // --- "Similar questions" live search -------------------------------
    // While the user types the title we query the existing (GIN-index backed)
    // `/search-questions/` endpoint and surface matching questions as links so
    // they can find an already-asked question before posting a duplicate.
    let similar = []; // matched questions
    let similarOpen = false; // dropdown visibility
    let similarLoading = false; // in-flight request
    let similarRequestId = 0; // guards against out-of-order responses
    let debounceTimer = null; // setTimeout handle for debouncing
    const SIMILAR_MIN_LEN = 3; // only search from 3 chars
    const SIMILAR_LIMIT = 10; // show a few links
    const SIMILAR_DEBOUNCE_MS = 300; // typing debounce

    async function searchSimilar() {
        // Read the current value directly from the input so a rapid paste
        // (which can fire `input` before Svelte flushes `bind:value`) is not
        // searched with a stale string.
        const el = document.getElementById('title');
        const q = (el ? el.value : title || '').trim();

        // Require a minimum length; otherwise hide the panel entirely.
        if (!browser || q.length < SIMILAR_MIN_LEN) {
            similar = [];
            similarOpen = false;
            return;
        }

        const requestId = ++similarRequestId;
        similarLoading = true;
        try {
            const response = await api.post('search-questions/', {
                search: q,
                questions_per_page: SIMILAR_LIMIT
            });
            // Ignore a stale response if the user kept typing meanwhile.
            if (requestId !== similarRequestId) return;

            const j = JSON.parse(await response.text());
            similar = (response.ok && j.data && j.data.questions) || [];
            similarOpen = similar.length > 0;
        } catch (e) {
            if (requestId === similarRequestId) {
                similar = [];
                similarOpen = false;
            }
        } finally {
            if (requestId === similarRequestId) {
                similarLoading = false;
            }
        }
    }

    // Debounced wrapper attached to the title input's `input` event.
    function onTitleInput() {
        clearTimeout(debounceTimer);
        debounceTimer = setTimeout(searchSimilar, SIMILAR_DEBOUNCE_MS);
    }

    // Hide the dropdown when focus leaves the panel (links + input), with a
    // short delay so clicking a result registers first.
    function closeSimilar() {
        setTimeout(() => {
            similarOpen = false;
        }, 150);
    }

    onDestroy(() => {
        clearTimeout(debounceTimer);
    });

    async function onSubmit() {
        if ($page.data.user) {
            if (contentValue.length < 20 || contentValue.length > 100000) {
                formError = 'Question length should be 20 to 1000000 Characters!';
                return;
            }
            if (tagList.length < 1) {
                formError = 'At least one tag should be supplied.';
                return;
            }
            if (tagList.length > 5) {
                formError = 'Too many tags.';
                return;
            }
            const duplicate = firstDuplicateTag(tagList);
            if (duplicate) {
                formError = `Duplicate tag: ${duplicate}`;
                return;
            }
            formError = '';

            if (browser) {
                let xsrf_token = getCookie('xsrf_token');
                const response = await api.post(
                    'create-question',
                    { title: title, description: contentValue, tag_list: tagList },
                    xsrf_token
                );

                let text = await response.text();
                let j = text ? JSON.parse(text) : {};

                if (response.status === 200 && j.data.id) {
                    let id = j.data.id;
                    let slug = j.data.slug;
                    await goto(`/questions/${id}/${slug}`);
                } else {
                    // Was `response.code === 400`, which is always undefined on a
                    // Response - so backend errors were silently swallowed. And
                    // `M.toast` is not defined anywhere in the app, so it threw
                    // a ReferenceError instead of showing anything.
                    formError = j.message || 'Something went wrong, try again!';
                }
            }
        } else {
            throw redirect(307, '/questions');
        }
    }

    // function for auto-completing tags
    async function ts() {
        if (browser) {
            let xsrf_token = getCookie('xsrf_token');
            let response = await api.post(
                'get-tags',
                {
                    tag: document.getElementById('tags').value
                },
                xsrf_token
            );
            response = JSON.parse(await response.text());
            if (response.data) {
                let tags = [];
                for (let i = 0; i < response.data.length; i++) {
                    tags.push(response.data[i].name);
                }
                return tags;
            } else {
                return [];
            }
        }
    }
</script>

<div class="row justify-content-center align-items-center" style="margin-top:20px">
    <div class="col-12">
        {#if formError}
            <div class="alert alert-danger" role="alert" id="askError">{formError}</div>
        {/if}
        <h4>Post your question for discussion</h4>
        <hr />
        <form on:submit|preventDefault={onSubmit}>
            <div>
                <label for="title" class="form-label">Summary</label>
                <input
                    bind:value={title}
                    label="Title"
                    class="form-control"
                    id="title"
                    type="text"
                    minlength="6"
                    maxlength="256"
                    style="min-width:100%"
                    role="combobox"
                    aria-autocomplete="list"
                    aria-expanded={similarOpen}
                    aria-controls="similarQuestions"
                    autocomplete="off"
                    on:input={onTitleInput}
                    on:blur={closeSimilar}
                />
                <div
                    id="similarQuestions"
                    role="listbox"
                    class="list-group"
                    hidden={!similarOpen}
                >
                    {#if similarLoading}
                        <div class="list-group-item text-muted">Searching…</div>
                    {:else}
                        {#each similar as q}
                            <a
                                class="list-group-item list-group-item-action"
                                role="option"
                                aria-selected="false"
                                href="/questions/{q.id}/{q.slug}"
                            >
                                {q.title}
                            </a>
                        {/each}
                    {/if}
                </div>
            </div>
            <Editor bind:markup bind:contentValue minlength={20} maxlength={100000} />
            <div style="margin:20px"></div>
            <Preview {markup} />
            <div style="margin:30px"></div>
            <Tags
                name={'tags'}
                bind:tags={tagList}
                addKeys={[9]}
                maxTags={5}
                allowPaste={true}
                allowDrop={true}
                splitWith={','}
                onlyUnique={true}
                removeKeys={[27, 8]}
                placeholder="Tags, tab to complete"
                allowBlur={true}
                disable={false}
                id={'tags'}
                minChars={1}
                autoComplete={ts}
                class="form-control"
            />
            <div class="b-wrapper">
                <button type="submit" class="btn btn-primary"> Ask </button>
            </div>
        </form>
        <div
            class="modal modal-dialog-centered"
            tabindex="-1"
            role="dialog"
            id="myForm"
            style="top:300px; display:none"
        >
            <div class="modal-dialog" role="document">
                <div class="modal-content">
                    <div class="modal-header">
                        <h5 class="modal-title">Image Upload</h5>
                    </div>
                    <div class="modal-body">
                        <form class="form-container" on:submit|preventDefault={onImageUpload}>
                            <h4>Uplaoad Image(Max 2MB)</h4>
                            <input
                                type="file"
                                name="image"
                                accept="image/*"
                                id="image"
                                alt="image"
                            /><br />

                            <div class="modal-footer">
                                <button type="submit" class="btn btn-primary">Upload</button>
                                <button
                                    type="button"
                                    class="btn btn-primary"
                                    on:click={() => {
                                        document.getElementById('myForm').style.display = 'none';
                                    }}>Close</button
                                >
                            </div>
                        </form>
                    </div>
                </div>
            </div>
        </div>
    </div>
</div>

<style>
    /* Similar-questions dropdown: cyan links */
    #similarQuestions {
        max-height: 260px;
        overflow-y: auto;
        border: 1px solid rgba(66, 133, 244, 0.4);
        border-radius: 0 0 0.375rem 0.375rem;
        margin-top: -1px;
        box-shadow: 0 0.5rem 1rem rgba(0, 0, 0, 0.15);
    }

    /* Bootstrap's `.list-group-item-action` sets its own link colour, so target
       the anchors explicitly to use the app's standard link blue (`.anchor`). */
    #similarQuestions a.list-group-item,
    #similarQuestions a.list-group-item:link,
    #similarQuestions a.list-group-item:visited {
        color: #4285f4;
        text-decoration: none;
        background-color: transparent;
        cursor: pointer;
    }

    #similarQuestions a.list-group-item:hover,
    #similarQuestions a.list-group-item:focus {
        color: #2f6ad9;
        background-color: rgba(66, 133, 244, 0.12);
    }

    #similarQuestions a.list-group-item:active {
        color: #1f4fae;
    }
</style>
