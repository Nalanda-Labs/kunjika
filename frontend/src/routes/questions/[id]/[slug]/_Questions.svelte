<script>
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import * as api from '$lib/api.js';
	import TagList from '$lib/TagList.svelte';
	import { page } from '$app/stores';
	import { parseMarkdown } from '../../../../lib/utils/editor/utils.editor';
	import Preview from '../../../../components/Editor/Preview.svelte';
	import getCookie from '../../../../lib/cookie';
	import { browser } from '$app/environment';
	import { closeForm, addImageURL } from '../../../../lib/utils/editor/utils.editor';

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
	let asked_ts = '';
	let vote_by_current_user = 0;
	$: vote_class1 = 'anchor';
	$: vote_class2 = 'anchor';

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
			vote_by_current_user = response.data.vote_by_current_user;

			if (image_url === '') {
				initials = username[0];
			}

			asked_ts = new Date(response.data.cat * 1000);
			let now = Date.now();

			shown_ts = Math.floor((now - asked_ts) / 1000);
			console.log(shown_ts);

			if (shown_ts >= 259200) {
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

			if (vote_by_current_user === 1) {
				vote_class1 = 'vote';
			} else if (vote_by_current_user === -1) {
				vote_class2 = 'vote';
			}
		} else if (response.status === 404) {
			goto('/404');
		}

		response = await api.post(`question/get-answers/${id}/?limit=${limit}`, {
			cat: asked_ts
		});

		response = JSON.parse(await response.text());

		if (response.data) {
			// these questions are actually answers. the question has already been
			// fetched in the first request.
			questions = response.data.questions;
			for (var i = 0; i < questions.length; i++) {
				if (questions[i].image_url === '') {
					questions[i].initials = response.data.questions[i].username[0];
				}
				let asked_ts = new Date(questions[i].cat * 1000);
				let now = Date.now();
				questions[i].description = parseMarkdown(response.data.questions[i].description);
				let shown_ts = Math.floor((now - asked_ts) / 1000);
				if (shown_ts >= 259200) {
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

				questions[i].vote_class1 = 'anchor';
				questions[i].vote_class2 = 'anchor';

				if (questions[i].vote_by_current_user === 1) {
					questions[i].vote_class1 = 'vote';
				} else if (questions[i].vote_by_current_user === -1) {
					questions[i].vote_class2 = 'vote';
				}
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

		let editor = document.getElementById('editor');
		if (editor.style.display === 'none') {
			editor.style.display = 'block';
		}

		editor.scrollIntoView(false);
	}

	function colorLink(elementID, count_by_user) {
		if (count_by_user === 1) {
			let voteElement = document.getElementById(`vu-${elementID}`);
			voteElement.classList.remove('anchor');
			voteElement.classList.add('vote');
		} else if (count_by_user === 0) {
			let voteUpElement = document.getElementById(`vu-${elementID}`);
			let voteDownElement = document.getElementById(`vd-${elementID}`);
			voteUpElement.classList.remove('vote');
			voteDownElement.classList.remove('vote');
			voteUpElement.classList.add('anchor');
			voteDownElement.classList.add('anchor');
		} else if (count_by_user === -1) {
			let voteElement = document.getElementById(`vd-${elementID}`);
			voteElement.classList.remove('anchor');
			voteElement.classList.add('vote');
		}
	}

	async function vote(vote, elementID) {
		if (browser) {
			let xsrf_token = getCookie('xsrf_token');
			const response = await api.post('votes', { vote, id: parseInt(elementID) }, xsrf_token);
			const j = JSON.parse(await response.text());

			if (response.status === 200 && j.success && 'count_by_user' in j) {
				if (elementID == id) {
					votes = vote + parseInt(votes);
					colorLink(elementID, j.count_by_user);
				} else {
					for (var i = 0; i < questions.length; i++) {
						if (questions[i].question_id == elementID) {
							questions[i].votes = vote + parseInt(questions[i].votes);
							questions = questions;
							colorLink(elementID, j.count_by_user);
							break;
						}
					}
				}
			} else {
				alert(j.data);
			}
		}
	}
	async function acceptAnswer(elementID) {
		if (browser) {
			let xsrf_token = getCookie('xsrf_token');
			const response = await api.post(`accept-answer/${id}/${elementID}/`, {}, xsrf_token);

			if (response.status === 200) {
				for (var i = 0; i < questions.length; i++) {
					let answerElement = document.getElementById(`a-${elementID}`);
					if (questions[i].question_id == elementID) {
						if (questions[i].answer_accepted == false) {
							questions[i].answer_accepted = true;
							answerElement.classList.add('answer');
						} else {
							questions[i].answer_accepted = false;
							answerElement.classList.remove('answer');
						}
					} else {
						questions[i].answer_accepted = false;
						answerElement.classList.remove('answer');
					}
				}
				questions = questions;
			} else {
				alert(response.message);
			}
		}
	}
</script>

<svelte:head>
	<title>{title}</title>
</svelte:head>
<div>
	<h3>{title}</h3>
	<hr />
	<div style="margin-top:10px">
		<div style="float:left;margin-right:10px;z-index:2;">
			{#if image_url === ''}
				<a href="/users/{posted_by}/{username}">
					<p data-letters={initials.toUpperCase()} />
				</a>
			{:else}
				<a href="/users/{posted_by}/{username}">
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
					<a href="/vote-up" class={vote_class1} id="vu-{id}" on:click|preventDefault={vote(1, id)}>
						<i class="fas fa-angle-up" />
					</a>
				{/if}
				<span style="text-align:center">{votes}</span>
				{#if $page.data.user && posted_by != $page.data.user.id}
					<a
						href="/vote-down"
						class={vote_class2}
						id="vd-{id}"
						on:click|preventDefault={vote(-1, id)}
					>
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
	{#each questions as { question_id, description, votes, posted_by_id, username, initials, image_url, shown_ts, answer_accepted, reply_to_id, rusername, rimage_url, vote_class1, vote_class2 }}
		<hr style="border-bottom:1px solid;color:#ccc;" />
		<div style="margin-top:10px" id={question_id} href="#{question_id}">
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
						<a
							href="/vote-up"
							class={vote_class1}
							id="vu-{question_id}"
							on:click|preventDefault={vote(1, question_id)}
						>
							<i class="fas fa-angle-up" />
						</a>
					{/if}
					<span style="text-align:center">{votes}</span>
					{#if $page.data.user && posted_by_id != $page.data.user.id}
						<a
							href="/vote-down"
							class={vote_class2}
							id="vd-{question_id}"
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
								class="anchor answer"
								id="a-{question_id}"
								on:click|preventDefault={acceptAnswer(question_id)}
							>
								<i class="fas fa-check" />
							</a>
						{:else}
							<a
								href="/accept-answer"
								class="anchor"
								id="a-{question_id}"
								on:click|preventDefault={acceptAnswer(question_id)}
							>
								<i class="fas fa-check" />
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
							><img
								src="{rimage_url}?s=32"
								alt={rusername}
								style="display:inline;margin-right:10px;width: 32px;"
							/></a
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
