<script>
	import { goto } from '$app/navigation';
	import * as api from '$lib/api.js';
	import { page } from '$app/stores';
	import { browser } from '$app/environment';
	import Tags from 'svelte-tags-input';
	import '../../../../highlight.css';
	import { redirect } from '@sveltejs/kit';
	import Editor from '../../../../../components/Editor/Editor.svelte';
	import Preview from '../../../../../components/Editor/Preview.svelte';
	import getCookie from '../../../../../lib/cookie.js';
	import '../../../../../editor.css';
    import { onMount } from "svelte";
    import { parseMarkdown } from "../../../../../lib/utils/editor/utils.editor";

	let title = "";
    let tagList = [];
    let time = "";
    let votes = 0;
    let posted_by;
    let username;
    let image_url = "";
    let initials = "";
    let shown_ts;
    let description = "";
    let value = "";
    let reply_to_id = "";
    let user_replied;
    let contentValue = '',
		markup = '';
	let showContentValueToast = false;


    onMount(async () => {
        let response = await api.get(`questions/${$page.params.id}/${$page.params.slug}`);

        if (response.status === 200) {
            response = JSON.parse(await response.text());
            title = response.data.title;
            console.log(title);
            description = response.data.description;
            value = parseMarkdown(response.data.description);
            contentValue = response.data.description;
            tagList = response.data.tags.map((tag) => tag);
            time = response.data.created_at;
            votes = response.data.votes;
            posted_by = response.data.posted_by_id;
            username = response.data.username;
            reply_to_id = response.data.posted_by_id;
            user_replied = username;
            image_url = response.data.image_url;
            if (image_url === "") {
                initials = username[0];
            }
            let asked_ts = Date.parse(time);
            let now = Date.now();
            shown_ts = Math.floor((now - asked_ts) / 1000);
            if (shown_ts >= 259200) {
                asked_ts = new Date(time);
                let year = asked_ts.getYear() + 1900;
                let month = asked_ts.getMonth() + 1;
                shown_ts = asked_ts.getDate() + "/" + month + "/" + year;
            } else if (172800 <= shown_ts && shown_ts < 259200) {
                shown_ts = "asked 2 days ago";
            } else if (86400 <= shown_ts && shown_ts < 172800) {
                shown_ts = "asked yesterday";
            } else if (3600 <= shown_ts && shown_ts < 8640000) {
                shown_ts = "asked " + Math.floor(shown_ts / 3600) + "h ago";
            } else if (60 <= shown_ts && shown_ts < 3600) {
                shown_ts = "asked " + Math.floor(shown_ts / 60) + "m ago";
            } else {
                shown_ts = "asked " + shown_ts + "s ago";
            }
        } else if (response.status === 404) {
            goto("/404");
        }
    })

	async function onSubmit() {
		if ($page.data.user) {
			if (contentValue.length < 20 || contentValue.length > 100000) {
				showContentValueToast = true;
			}
			if (tagList.length < 1) {
				// Toast('At least one tag should be supplied.');
			}

			if (browser) {
				let xsrf_token = getCookie('xsrf_token');
				const response = await api.post(
					`edit-post/${$page.params.id}`,
					{ title: title, description: contentValue, tag_list: tagList },
					xsrf_token
				);

				let text = await response.text();
				let j = text ? JSON.parse(text) : {};
				console.log(j);

				if (response.status === 200 && j.data.id) {
					let id = j.data.id;
					let slug = j.data.slug;
					await goto(`/questions/${id}/${slug}`);
				} else if (response.code === 400) {
					M.toast({ html: j.message });
				}
			}
		} else {
			throw redirect(307, '/questions');
		}
	}

	// function handleTags(event) {
	// 	tagList = event.detail.tags;
	// 	console.log(tagList);
	// 	let re = /[a-zA-Z0-0\-\+]+/;
	// 	for (let i = 0; i < tagList.length; i++) {
	// 		if (tagList[i].length > 32) {
	// 			document.getElementById('tags-helper').innerHTML = '32 Characterx max.';
	// 			document.getElementById('tags-helper').style.color = '#800';
	// 			break;
	// 		} else {
	// 			document.getElementById('tags-helper').innerHTML = '';
	// 		}
	// 	}
	// }

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
	<div class="col s12 m8 offset-m4 xl10 offset-xl2">
		{#if showContentValueToast}
			<Toast>Question length should be 20 to 1000000 Characters!</Toast>
		{/if}
		<h4>Post your question for discussion</h4>
		<hr />
		<form class="col s12" on:submit|preventDefault={onSubmit}>
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
			<Editor bind:markup bind:contentValue minlength={20} maxlength={100000} />
			<Preview {markup} />
			<div style="margin:30px" />
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
			<div class="b-wrapper">
				<button type="submit" class="btn"> Ask </button>
			</div>
		</form>
	</div>
</div>
