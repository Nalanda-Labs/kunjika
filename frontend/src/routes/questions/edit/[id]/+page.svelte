<script>
    import { goto } from '$app/navigation';
    import * as api from '$lib/api.js';
    import { page } from '$app/stores';
    import { browser } from '$app/environment';
    import Tags from 'svelte-tags-input';
    import '../../../highlight.css';
    import { redirect } from '@sveltejs/kit';
    import Editor from '../../../../components/Editor/Editor.svelte';
    import Preview from '../../../../components/Editor/Preview.svelte';
    import getCookie from '../../../../lib/cookie.js';
    import '../../../../editor.css';
    import { onMount } from 'svelte';
    import { parseMarkdown } from '../../../../lib/utils/editor/utils.editor';
    import onImageUpload from '../../../../lib/imageUpload';
    import { firstDuplicateTag } from '../../../../lib/utils/tags.js';

    let title = '';
    let tagList = [];
    let time = '';
    let votes = 0;
    let posted_by;
    let username;
    let image_url = '';
    let initials = '';
    let shown_ts;
    let description = '';
    let value = '';
    let reply_to_id = '';
    let user_replied;
    let contentValue = '',
        markup = '';
    let formError = '';
    let notAuthor = false;

    onMount(async () => {
        let xsrf_token = getCookie('xsrf_token');
        let response = await api.get(`edit/question/${$page.params.id}`, xsrf_token);

        if (response.status === 200) {
            response = JSON.parse(await response.text());
            title = response.data.title;
            description = response.data.description;
            value = parseMarkdown(response.data.description);
            contentValue = response.data.description;
            tagList = response.data.tags.map((tag) => tag);
            console.log(response.data.tags);
            time = response.data.created_at;
            votes = response.data.votes;
            posted_by = response.data.posted_by_id;
            // Only the author may edit; the API enforces this too, but the
            // form should not be offered at all.
            notAuthor = !($page.data.user && posted_by === $page.data.user.id);
            username = response.data.username;
            reply_to_id = response.data.posted_by_id;
            user_replied = username;
            image_url = response.data.image_url;
            if (image_url === '') {
                initials = username[0];
            }
            let asked_ts = Date.parse(time);
            let now = Date.now();
            shown_ts = Math.floor((now - asked_ts) / 1000);
            if (shown_ts >= 259200) {
                asked_ts = new Date(time);
                let year = asked_ts.getYear() + 1900;
                let month = asked_ts.getMonth() + 1;
                shown_ts = asked_ts.getDate() + '/' + month + '/' + year;
            } else if (172800 <= shown_ts && shown_ts < 259200) {
                shown_ts = 'asked 2 days ago';
            } else if (86400 <= shown_ts && shown_ts < 172800) {
                shown_ts = 'asked yesterday';
            } else if (3600 <= shown_ts && shown_ts < 8640000) {
                shown_ts = 'asked ' + Math.floor(shown_ts / 3600) + 'h ago';
            } else if (60 <= shown_ts && shown_ts < 3600) {
                shown_ts = 'asked ' + Math.floor(shown_ts / 60) + 'm ago';
            } else {
                shown_ts = 'asked ' + shown_ts + 's ago';
            }
        } else if (response.status === 404) {
            goto('/404');
        }
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
                    `edit-post/${$page.params.id}`,
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
                    // Was `response.code === 400`, always undefined on a Response, so
                    // backend errors were swallowed. And `M.toast` is not defined
                    // anywhere in the app, so it threw a ReferenceError. The API now
                    // answers 403 for someone else's post, so surface that too.
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

<div class="row">
    <div class="col s12 m12 xl10 offset-xl2">
        {#if formError}
            <div class="alert alert-danger" role="alert" id="editError">{formError}</div>
        {/if}
        {#if notAuthor}
            <div class="alert alert-warning" role="alert" id="editNotAuthor">
                You can only edit your own post.
            </div>
        {/if}
        <h4>Post your question for discussion</h4>
        <hr />
        {#if !notAuthor}
            <form class="col s12" on:submit|preventDefault={onSubmit}>
                {#if title}
                    <div class="input-field">
                        <input
                            bind:value={title}
                            label="Title"
                            id="title"
                            type="text"
                            minlength="6"
                            maxlength="256"
                            style="min-width:100%"
                        />
                        <label for="title">Summary</label>
                    </div>
                {/if}
                <Editor bind:markup bind:contentValue minlength={20} maxlength={100000} />
                <Preview {markup} />
                {#if tagList.length}
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
                    />
                {/if}
                <div class="b-wrapper">
                    <button type="submit" class="btn btn-primary">Edit</button>
                </div>
            </form>
        {/if}
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
