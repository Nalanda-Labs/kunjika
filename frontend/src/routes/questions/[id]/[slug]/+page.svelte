<script context="module">
	export function preload({ params }, { user }) {}
</script>

<script>
	import Questions from './_Questions.svelte';
	import { page } from '$app/stores';
	import { onMount } from 'svelte';
	import * as api from '$lib/api.js';
	import Editor from '../../../../components/Editor/Editor.svelte';
	import '../../../../editor.css';
	import Preview from '../../../../components/Editor/Preview.svelte';
	import { browser } from '$app/environment';
	import getCookie from '../../../../lib/cookie';

	let reply_to_id;
	let user_replied;
	let image_url;
	let value = '';
	let topic = {};
	let questions = [];
	let inProgress = false;
	const id = $page.params.id;
	const slug = $page.params.slug;
	let markup = '',
		contentValue = '';
	let xsrf_token = '';

	function show_editor(reply_to, username) {
		reply_to_id = reply_to;
		user_replied = username;
		if (document.getElementById('editor').style.display === 'none') {
			document.getElementById('editor').style.display = 'block';
		} else {
			document.getElementById('editor').style.display = 'none';
		}
		// TODO: Fix scroll on showing editor
		var editorDiv = document.getElementById('editor');
		editorDiv.scrollTo({ left: 0, top: document.body.scrollHeight, behavior: 'smooth' });
	}
	async function reply() {
		if ($page.data.user) {
			inProgress = true;
			value = contentValue;
			if (value.length < 20 || value.length > 100000) {
				M.toast({
					html: 'Body should not be less than 20 or more than 100000 characters.'
				});
				return;
			}

			if (browser) {
				xsrf_token = getCookie('xsrf_token');
			}

			let reply_to = reply_to_id;

			let response = await api.post(`answer`, { id, value, reply_to }, xsrf_token);
			response = JSON.parse(await response.text());
			if (response.data) {
				document.getElementById('editor').style.display = 'none';
				const l = questions.length;
				questions[l] = {
					question_id: response.data,
					description: value,
					votes: 0,
					posted_by: $page.data.user.id,
					username: $page.data.user.username,
					image_url: $page.data.user.image_url,
					shown_ts: '0 s',
					initials: $page.data.user.username[0],
					answer_accepted: false
				};
				questions = questions;
			}
			inProgress = false;
		} else {
			alert('You are not logged in.');
		}
	}

	function handleChange(e) {
		value = e.detail.value;
	}
</script>

<svelte:head>
	<title />
</svelte:head>

<div class="row justify-content-center align-items-center" style="margin-top:20px">
	<div class="col-8">
		<div class="col-xs-12">
			<Questions {id} {slug} bind:reply_to_id bind:questions bind:user_replied />
			<div id="questions-end" style="display:none" />
			<hr style="border-bottom:1px solidl;color:#eee" />
			{#if $page.data.user}
				<a href="/edit/{id}" class="anchor" title="Edit your post" style="margin-right:5px"
					><span class="material-icons" style="vertical-align:bottom">edit</span> Edit</a
				>
				<!-- <a
                    href="/report/{id}"
                    class="anchor danger"
                    title="Report abusive or inappropriate content"
                    style="margin-right:5px"
                    on:click="{notImplemented}"
                    ><span class="material-icons" style="vertical-align:bottom"
                        >report</span
                    >Report</a
                > -->
				<!-- <a
                    href="/share/{id}"
                    class="anchor"
                    title="Share a link to this post"
                    style="margin-right:5px"
                    ><span class="material-icons" style="vertical-align:bottom"
                        >share</span
                    >Share</a
                > -->
				<!-- <a
                    href="/bookmark/{id}"
                    class="anchor"
                    title="Bookmark this post"
                    style="margin-right:5px"
                    on:click="{notImplemented}"
                    ><span class="material-icons" style="vertical-align:bottom"
                        >bookmark</span
                    >Bookmark</a
                > -->
				<a
					href="/reply"
					on:click|preventDefault={show_editor(reply_to_id, user_replied)}
					class="anchor"
					title="Reply to this post"
					style="margin-right:5px"
					><span class="material-icons" style="vertical-align:bottom">reply</span>Reply</a
				>
			{/if}
			<form on:submit|preventDefault={reply}>
				<div id="editor" style="display:none;margin-top:10px">
					<span style="color:#4285F4">Replying to @{user_replied}</span>
					<Editor bind:markup bind:contentValue minlength={20} maxlength={100000} />
					<Preview {markup} />
					<div class="b-wrapper">
						<button class="btn"> Reply </button>
					</div>
				</div>
				<div style="min-height: 20px;" />
			</form>
		</div>
	</div>
</div>
