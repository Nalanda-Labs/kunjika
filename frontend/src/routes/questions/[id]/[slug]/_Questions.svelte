<script>
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import * as api from '$lib/api.js';
	// import "bytemd/dist/index.min.css";
	import TagList from '$lib/TagList.svelte';
	import { page } from '$app/stores';
	import { parseMarkdown } from '../../../../lib/utils/editor/utils.editor';
	import Preview from '../../../../components/Editor/Preview.svelte';
	import getCookie from '../../../../lib/cookie';
	import { browser } from '$app/environment';
	import ListErrors from '../../../../lib/ListErrors.svelte';

	export let id;
	export let slug;
	export let reply_to_id;
	export let questions;
	export let user_replied;
	let value = '';

	let title = '';
	let taglist = [];
	let offset = 0;
	let limit = 50;
	let count = 0;
	let time = '';
	let votes = 0;
	let posted_by;
	let username;
	let image_url = '';
	let initials = '';
	let shown_ts;
	let description = '';
	let errors = '';

	onMount(async () => {
		let response = await api.get(`questions/${id}/${slug}`);

		if (response.status === 200) {
			response = JSON.parse(await response.text());
			title = response.data.title;
			description = response.data.description;
			value = parseMarkdown(response.data.description);
			taglist = response.data.tags.map((tag) => tag);
			time = response.data.created_at;
			votes = response.data.votes;
			posted_by = response.data.posted_by_id;
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

		response = await api.get(`question/get-answers/${id}/?time=${time}&limit=${limit}`);

		response = JSON.parse(await response.text());

		if (response.data) {
			// these questions are actually answers. the question has already been
			// fetched in the first request.
			questions = response.data.questions;
			for (var i = 0; i < questions.length; i++) {
				if (questions[i].image_url === '') {
					questions[i].initials = response.data.questions[i].username[0];
				}
				let asked_ts = Date.parse(questions[i].created_at);
				let now = Date.now();
				questions[i].		description = parseMarkdown(response.data.questions[i].description);
				let shown_ts = Math.floor((now - asked_ts) / 1000);
				if (shown_ts >= 259200) {
					asked_ts = new Date(questions[i].created_at);
					let year = asked_ts.getYear() + 1900;
					let month = asked_ts.getMonth() + 1;
					shown_ts = asked_ts.getDate() + '/' + month + '/' + year;
				} else if (172800 <= shown_ts && shown_ts < 259200) {
					shown_ts = '2 days ago';
				} else if (86400 <= shown_ts && shown_ts < 172800) {
					shown_ts = 'yesterday';
				} else if (3600 <= shown_ts && shown_ts < 8640000) {
					shown_ts = Math.floor(shown_ts / 3600) + 'h';
				} else if (60 <= shown_ts && shown_ts < 3600) {
					shown_ts = Math.floor(shown_ts / 60) + 'm';
				} else {
					shown_ts = shown_ts + 's';
				}
				questions[i].shown_ts = shown_ts;
			}
			offset += limit;
			if (response.data.questions.length) {
				time = response.data.questions[response.data.questions.length - 1].created_at;
			}
		}
	});

	function show_editor(reply_to, username) {
		reply_to_id = reply_to;
		user_replied = username;
		if (document.getElementById('editor').style.display === 'none') {
			document.getElementById('editor').style.display = 'block';
		} else {
			document.getElementById('editor').style.display = 'none';
		}
	}

	async function vote(vote, elementID) {
		if (browser) {
			let xsrf_token = getCookie('xsrf_token');
			const response = await api.post('votes', { vote, id: parseInt(elementID) }, xsrf_token);
			const j = JSON.parse(await response.text());

			if (response.status === 200 && j.data) {
				if (elementID == id) {
					votes = vote + parseInt(votes);
				} else {
					for (var i = 0; i < questions.length; i++) {
						if (questions[i].question_id == elementID) {
							questions[i].votes = vote + parseInt(questions[i].votes);
							questions = questions;
							break;
						}
					}
				}
			} else {
				let j = JSON.parse(await response.text());
				errors = j.message;
			}
		}
	}
	async function acceptAnswer(elementID) {
		const response = await api.post(
			`accept-answer/${id}/${elementID}/`,
			{},
			$page.data.user.xsrf_token
		);

		if (response.data.code != 200) {
			M.toast({ html: response.data.msg });
		} else {
			for (var i = 0; i < questions.length; i++) {
				if (questions[i].question_id == elementID) {
					if (questions[i].answer_accepted == false) {
						questions[i].answer_accepted = true;
					} else {
						questions[i].answer_accepted = false;
					}
				} else {
					questions[i].answer_accepted = false;
				}
			}
			questions = questions;
		}
	}
</script>

<svelte:head>
	<title>{title}</title>
</svelte:head>
<div>
	<ListErrors {errors} />
	<h3>{title}</h3>
	<hr />
	<div style="margin-top:10px">
		<div style="float:left;margin-right:10px;z-index:2;">
			{#if image_url === ''}
				<a href="/user/{posted_by}/{username}">
					<p data-letters={initials.toUpperCase()} />
				</a>
			{:else}
				<a href="/user/{posted_by}/{username}">
					<img
						src={image_url}
						alt="profile pic"
						style="width: 3.5em;height: 3.5em;line-height: 3.5em;text-align: center;"
					/>
				</a>
			{/if}
			<br />
			<div style="text-align: center;font-size: 24px">
				{#if $page.data.user && posted_by != $page.data.user.id}
					<a href="/vote-up" class="anchor" on:click|preventDefault={vote(1, id)}>
						<i class="fas fa-angle-up" />
					</a>
				{/if}
				<span style="text-align:center">{votes}</span>
				{#if $page.data.user && posted_by != $page.data.user.id}
					<a href="/vote-down" class="anchor" on:click|preventDefault={vote(-1, id)}>
						<i class="fas fa-angle-down" />
					</a>
				{/if}
			</div>
		</div>
		<div style="float:left; position:relative;width:calc(100% - 70px)">
			<span style="font-weight:bold;color:#888;display:inline!important">{username}</span><span
				style="float:right"
			>
				{shown_ts}</span
			>
			<!-- <span style="float:left">{shown_ts}</span> -->
			<div style="margin:10px" />
			<div class="answers">
				<Preview markup={value} />
			</div>
			<TagList {taglist} />
			{#if $page.data.user}
				<div style="float:right">
					<a
						href="/questions/edit/{id}"
						class="anchor"
						title="Edit your post"
						style="margin-right:5px;display:inline"
						><span class="material-icons" style="vertical-align:bottom">edit</span> Edit</a
					>
					<a
						href="/report/{id}"
						class="anchor danger"
						title="Report abusive or inappropriate content"
						style="margin-right:5px"
						><span class="material-icons" style="vertical-align:bottom">report</span>Report</a
					>
					<a
						href="/share/{id}"
						class="anchor"
						title="Share a link to this post"
						style="margin-right:5px"
						><span class="material-icons" style="vertical-align:bottom">share</span>Share</a
					>

					<a
						href="/bookmark/{id}"
						class="anchor"
						title="Bookmark this post"
						style="margin-right:5px"
						><span class="material-icons" style="vertical-align:bottom">bookmark</span>Bookmark</a
					>
					<a
						href="/reply"
						on:click|preventDefault={show_editor(posted_by, username)}
						class="anchor"
						title="Reply to this post"
						style="margin-right:5px;display:inline"
						><span class="material-icons" style="vertical-align:bottom">reply</span>Reply</a
					>
				</div>
			{/if}
		</div>
	</div>
	<div style="clear:both;margin-bottom:10px" />
	{#each questions as { question_id, description, votes, posted_by_id, username, initials, image_url, shown_ts, answer_accepted, reply_to_id, rusername, rimage_url }}
		<hr style="border-bottom:1px solid;color:#ccc;" />
		<div style="margin-top:10px" id="{question_id}">
			<div style="float:left;margin-right:10px">
				{#if image_url === '' || image_url === undefined}
					<a href="/users/{posted_by_id}/{username}">
						<p data-letters={initials.toUpperCase()} />
					</a>
				{:else}
					<a href="/users/{posted_by_id}/{username}">
						<img
							src={image_url}
							alt="profile pic"
							style="width: 3.5em;height: 3.5em;line-height: 3.5em;text-align: center;"
						/>
					</a>
				{/if}
				<br />
				<div style="text-align: center;font-size: 24px">
					{#if $page.data.user && posted_by_id != $page.data.user.id}
						<a href="/vote-up" class="anchor" on:click|preventDefault={vote(1, question_id)}>
							<i class="fas fa-angle-up" />
						</a>
					{/if}
					<span style="text-align:center">{votes}</span>
					{#if $page.data.user && posted_by_id != $page.data.user.id}
						<a
							href="/vote-down"
							class="anchor"
							style="display:block"
							on:click|preventDefault={vote(-1, question_id)}
						>
							<i class="fas fa-angle-down" />
						</a>
					{/if}
					{#if $page.data.user && posted_by == $page.data.user.id}
						{#if answer_accepted}
							<a
								href="/accept-answer"
								class="anchor"
								on:click|preventDefault={acceptAnswer(question_id)}
							>
								<i class="fas fa-check" style="color: #080" />
							</a>
						{:else}
							<a
								href="/accept-answer"
								class="anchor"
								on:click|preventDefault={acceptAnswer(question_id)}
							>
								<i class="fas fa-check" style="color: #ddd" />
							</a>
						{/if}
					{/if}
				</div>
			</div>
			<div style="float:left; position:relative;width:calc(100% - 70px)">
				<span style="display:inline;font-weight:bold;color:#888">{username}</span>
				<span style="float:right">posted {shown_ts}</span>
				{#if reply_to_id}
					<span style="float:right"
						>reply to <a href="/users/{reply_to_id}/{rusername}"
							><img src="{rimage_url}?s=32" alt={rusername} style="display:inline;margin-right:10px" /></a
						></span
					>
				{/if}
				<div style="margin:10px" />
				<div class="answers">
					<Preview markup={description} />
				</div>
				{#if $page.data.user}
					<div style="float:right">
						<a
							href="/questions/edit/{question_id}"
							class="anchor"
							title="Edit your post"
							style="margin-right:5px"
							><span class="material-icons" style="vertical-align:bottom">edit</span> Edit</a
						>
						<a
							href="/report/{question_id}"
							class="anchor danger"
							title="Report abusive or inappropriate content"
							style="margin-right:5px"
							><span class="material-icons" style="vertical-align:bottom">report</span>Report</a
						>
						<a
							href="/reply"
							on:click|preventDefault={show_editor(posted_by, username)}
							class="anchor"
							title="Reply to this post"
							style="margin-right:5px"
							><span class="material-icons" style="vertical-align:bottom">reply</span>Reply</a
						>
						<a
							href="/share/{question_id}"
							class="anchor"
							title="Share a link to this post"
							style="margin-right:5px"
							><span class="material-icons" style="vertical-align:bottom">share</span>Share</a
						>
						<a
							href="/bookmark/{question_id}"
							class="anchor"
							title="Bookmark this post"
							style="margin-right:5px"
							><span class="material-icons" style="vertical-align:bottom">bookmark</span>Bookmark</a
						>
					</div>
				{/if}
			</div>
		</div>
		<div style="clear:both" />
	{/each}
</div>

<style>
	[data-letters]:before {
		content: attr(data-letters);
		display: inline-block;
		font-size: 1.5em;
		width: 2.5em;
		height: 2.5em;
		line-height: 2.5em;
		text-align: center;
		border-radius: 50%;
		background: #0f9d58;
		vertical-align: middle;
		color: white;
		margin-top: -10px;
	}
	p {
		font-weight: 300;
	}
</style>
