<script context="module">
    export function preload({ params }, { user }) {}
</script>

<script>
    import Questions from "./_Questions.svelte";
    import { stores, page } from "$app/stores";
    import { onMount } from "svelte";
    import * as api from "$lib/api.js";

    let reply_to_id;
    let user_replied;
    let image_url;
    let value = "";
    let Editor = null;
    let topic = {};
    let questions = [];
    const id = $page.params.id;
    const slug = $page.params.slug;

    const { session } = stores();

    function show_editor(reply_to, username) {
        reply_to_id = reply_to;
        user_replied = username;
        if (document.getElementById("editor").style.display === "none") {
            document.getElementById("editor").style.display = "block";
        } else {
            document.getElementById("editor").style.display = "none";
        }
        // TODO: Fix scroll on showing editor
        // var editorDiv = document.getElementById("container");
        // console.log(editorDiv.scrollHeight, editorDiv.scrollTop);
        // editorDiv.scrollTop = editorDiv.scrollHeight;
        // console.log(editorDiv.scrollHeight, editorDiv.scrollTop);
    }
    async function reply() {
        if ($session.user) {
            inProgress = true;
            if (value.length < 20 || value.length > 100000) {
                M.toast({
                    html: "Body should not be less than 20 or more than 100000 characters.",
                });
                return;
            }

            let reply_to = reply_to_id;

            const response = await api.post(
                `create-post?topic_id=${id}`,
                { value, reply_to },
                localStorage.getItem("jwt")
            );

            if (response.post_id) {
                document.getElementById("editor").style.display = "none";
                const l = questions.length;
                questions[l] = {
                    topic_id: response.post_id,
                    description: value,
                    votes: 0,
                    posted_by: $session.user_id,
                    username: $session.user,
                    image_url: response.image_url,
                    shown_ts: "0 s",
                    initials: $session.user[0],
                };
            }
            inProgress = false;
        } else {
            M.toast({ html: "You are not logged in." });
        }
    }
    onMount(async () => {
        const bytemd = await import("bytemd");
        Editor = bytemd.Editor;
    });

    function handleChange(e) {
        value = e.detail.value;
    }
</script>

<svelte:head>
    <title />
</svelte:head>

<div class="topic" id="container">
    <div class="row">
        <div class="col-xs-12">
            <Questions
                {id}
                {slug}
                bind:reply_to_id
                bind:questions
                bind:topic
                bind:user_replied
            />
            <div id="questions-end" style="display:none" />
            <hr style="border-bottom:1px solidl;color:#eee" />
            {#if $session.user}
                <a
                    href="/edit/{id}/{slug}"
                    class="anchor"
                    title="Edit your post"
                    style="margin-right:5px"
                    ><span class="material-icons" style="vertical-align:bottom"
                        >edit</span
                    > Edit</a
                >
                <a
                    href="/report/{id}"
                    class="anchor danger"
                    title="Report abusive or inappropriate content"
                    style="margin-right:5px"
                    ><span class="material-icons" style="vertical-align:bottom"
                        >report</span
                    >Report</a
                >
                <a
                    href="/share/{id}"
                    class="anchor"
                    title="Share a link to this post"
                    style="margin-right:5px"
                    ><span class="material-icons" style="vertical-align:bottom"
                        >share</span
                    >Share</a
                >
                <a
                    href="/bookmark/{id}"
                    class="anchor"
                    title="Bookmark this post"
                    style="margin-right:5px"
                    ><span class="material-icons" style="vertical-align:bottom"
                        >bookmark</span
                    >Bookmark</a
                >
                <a
                    href="/reply"
                    on:click|preventDefault={show_editor(
                        reply_to_id,
                        user_replied
                    )}
                    class="anchor"
                    title="Reply to this post"
                    style="margin-right:5px"
                    ><span class="material-icons" style="vertical-align:bottom"
                        >reply</span
                    >Reply</a
                >
            {/if}
            <form on:submit|preventDefault={reply}>
                <div id="editor" style="display:none;margin-top:10px">
                    <span style="color:#4285F4"
                        >Replying to @{user_replied}</span
                    >
                    <svelte:component
                        this={Editor}
                        on:change={handleChange}
                        mode="tab"
                        {value}
                    />
                    <div class="b-wrapper">
                        <button class="btn"> Reply </button>
                    </div>
                </div>
                <div style="min-height: 20px;" />
            </form>
        </div>
    </div>
</div>

<style>
    @media (max-width: 720px) {
        .topic {
            width: 100%;
        }
    }
    @media (max-width: 4096px) {
        .topic {
            width: 800px;
        }
        :global(.bytemd-editor .CodeMirror) {
            height: 90% !important;
        }
    }
</style>
