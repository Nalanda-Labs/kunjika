<script>
	import * as api from '$lib/api.js';
	import { page } from '$app/stores';
	import { browser } from '$app/environment';
	import Tags from 'svelte-tags-input';
	import { Label, Textarea, Input, Helper } from 'flowbite-svelte';
	import Markdoc from '@markdoc/markdoc';
	import hljs from 'highlight.js';
	import '../highlight.css';
	import { redirect } from '@sveltejs/kit';

	let title = '';
	let description = '';
	let tagList = [];

	function process() {
		const config = { variables: { version: '1.0' } };
		const ast = Markdoc.parse(description);
		const content = Markdoc.transform(ast, config);
		const html = Markdoc.renderers.html(content);

		document.getElementById('preview').innerHTML = html;
		document.querySelectorAll('pre').forEach((el) => {
    		hljs.highlightElement(el);
  		});
	}

	async function onSubmit() {
		console.log(title, description, tagList);
	    if ($page.data.user) {
	        if (title < 6 || title > 256) {
	            M.toast({
	                html: "Title should not be less than 6 or more than 256 characters.",
	            });
	            return;
	        }
	        if (value.length < 20 || value.length > 100000) {
	            M.toast({
	                html: "question should not be less than 20 or more than 100000 characters.",
	            });
	            return;
	        }

	        body = description;

	        if (question.tagList.length < 1) {
	            M.toast({ html: "At least one tag should be supplied." });
	        }

	        const response = await api.post(
	            "create-question",
	            { "title": question.title, "description": question.body, "tag_list": question.tagList },
	            $page.data.user.xsrf_token
	        );

	        if (response.code === 200 && response.data.id && response.data.slug) {
	            id = response.data.id;
	            await goto(`/questions/${id}`);
	        } else if(response.code === 400) {
	            M.toast({html: response.msg});
	        }
	    } else {
	        throw redirect(307, '/questions');
	    }
	}

	function handleTags(event) {
		tagList = event.detail.tags;
		console.log(tagList);
		let re = /[a-zA-Z0-0\-\+]+/;
		for (let i = 0; i < tagList.length; i++) {
			if (tagList[i].length > 32) {
				document.getElementById('tags-helper').innerHTML = '32 Characterx max.';
				document.getElementById('tags-helper').style.color = '#800';
				break;
			} else {
				document.getElementById('tags-helper').innerHTML = '';
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

<main class="overflow-hidden relative m-2 p-4 question">
	<h4>Post your question for discussion</h4>
	<hr />
	<form on:submit|preventDefault={onSubmit}>
		<Label for="large-input" class="block mb-2">Title</Label>
		<Input
			size="lg"
			required
			bind:value={title}
			id="title"
			minlength="6"
			maxlength="256"
			name="title"
		/>
		<!-- <svelte:component this={Editor} on:change={handleChange} mode="tab" {value} /> -->
		<Label for="description" class="mb-2">Description</Label>
		<Textarea
			id="description"
			placeholder=""
			rows="4"
			name="body"
			bind:value={description}
			on:keyup={process}
		/>
		<div style="margin-top:20px" />
		<div id="preview" />
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
		<Helper class="text-sm mt-2" id="tags-helper" />
		<div class="b-wrapper">
			<button
				type="submit"
				class="w-full bg-primary-600 hover:bg-primary-700 text-white font-medium rounded-lg text-sm px-5 py-2.5 text-center"
			>
				Ask
			</button>
		</div>
	</form>
</main>

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
	#preview {
		border: 1px dashed;
		padding: 5px;
	}
</style>
