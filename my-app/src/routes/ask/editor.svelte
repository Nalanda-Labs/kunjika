<script>
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import * as api from '$lib/api.js';
	import { onMount } from 'svelte';
	import 'bytemd/dist/index.min.css';
	import { browser } from '$app/environment';
	import { Editor, rootCtx, defaultValueCtx } from '@milkdown/core';
	import { commonmark } from '@milkdown/preset-commonmark';
	import { nord } from '@milkdown/theme-nord';
	import { Button } from 'flowbite-svelte';
    import Tags from "svelte-tags-input";

	export let question;
	export const id = 876;
	let user = null;

	// async function onSubmit() {
	//     if ($page.data.user) {
	//         if (question.title < 6 || question.title > 256) {
	//             M.toast({
	//                 html: "Title should not be less than 6 or more than 256 characters.",
	//             });
	//             return;
	//         }
	//         if (value.length < 20 || value.length > 100000) {
	//             M.toast({
	//                 html: "question should not be less than 20 or more than 100000 characters.",
	//             });
	//             return;
	//         }

	//         question.body = value;

	//         if (question.tagList.length < 1) {
	//             M.toast({ html: "At least one tag should be supplied." });
	//         }

	//         const response = await api.post(
	//             "create-question",
	//             { "title": question.title, "description": question.body, "tag_list": question.tagList },
	//             $page.data.user.xsrf_token
	//         );

	//         if (response.code === 200 && response.data.id && response.data.slug) {
	//             id = response.data.id;
	//             await goto(`/questions/${id}`);
	//         } else if(response.code === 400) {
	//             M.toast({html: response.msg});
	//         }
	//     } else {
	//         M.toast({ html: "You are not logged in." });
	//     }
	// }

	// let Editor = null;
	// let plugins = null;
	// let value = '';

	// onMount(async () => {
	// 	if ($page.data.user) {
	// 		const bytemd = await import('bytemd');
	// 		Editor = bytemd.Editor;
	// 		Tags = (await import('$lib/Tags.svelte')).default;
	// 	} else {
	// 		goto('/login');
	// 	}
	// });

	// function handleChange(e) {
	// 	value = e.detail.value;
	// }

	function editor() {
		// to obtain the editor instance we need to store a reference of the editor.
		const MakeEditor = Editor.make()
			.config((ctx) => {
				ctx.set(rootCtx, document.querySelector('#editor'));
			})
			.config(nord)
			.use(commonmark)
			.create();
		MakeEditor.then((editor) => {
			// here you have access to the editor instance.
			// const exampleContent = "# Hello World!";
			// editor.action(replaceAll(exampleContent));
		});
	}

	function handleTags(event) {
		question.tagList = event.detail.tags;
		let re = /[a-zA-Z0-0\-\+]+/;
		for (let i = 0; i < question.tagList.length; i++) {
			if (question.tagList[i].length > 32) {
				M.toast({ html: '32 Characterx max.' });
			}
		}
	}

	function getCookie(cname) {
		let name = cname + '=';
		let decodedCookie = decodeURIComponent(document.cookie);
		let ca = decodedCookie.split(';');
		for (let i = 0; i < ca.length; i++) {
			let c = ca[i];
			while (c.charAt(0) == ' ') {
				c = c.substring(1);
			}
			if (c.indexOf(name) == 0) {
				return c.substring(name.length, c.length);
			}
		}
		return '';
	}
	// function for auto-completing tags
	async function ts() {
        console.log('hello');
		if (browser) {
			let xsrf_token = getCookie('xsrf_token');
            console.log(xsrf_token);
			const response = await api.post(
				'get-tags',
				{
					tag: document.getElementById('tags').value
				},
				xsrf_token
			);
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

<div class="question">
	<h4>Post your question for discussion</h4>
	<hr />
	<form method="post">
		<div class="input-field">
			<input
				bind:value={question.title}
				label="Title"
				id="title"
				type="text"
				minlength="6"
				maxlength="256"
				style="min-width:100%"
			/>
			<label for="title">Summary</label>
		</div>
		<!-- <svelte:component this={Editor} on:change={handleChange} mode="tab" {value} /> -->
        <!-- <div use:editor id="editor"/> -->
		<div style="margin:30px" />
		<Tags
			name={'tags'}
			on:tags={handleTags}
			addKeys={[9]}
			maxTags={5}
			allowPaste={true}
			allowDrop={true}
			splitWith={','}
			onlyUnique={true}
			removeKeys={[27, 8]}
			placeholder='Tags, tab to complete'
			allowBlur={true}
			disable={false}
			id={'tags'}
			minChars={1}
			autoComplete={ts}
		/>
		<div class="b-wrapper">
			<Button> Ask </Button>
		</div>
	</form>
</div>

<style>
	@media (max-width: 720px) {
		.question {
			width: 100%;
		}
	}
	@media (max-width: 4096px) {
		.question {
			width: 800px;
		}
	}
</style>
