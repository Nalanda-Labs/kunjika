<script context="module">
	export function preload({ params }, { user }) {}
</script>

<script>
	import { page } from '$app/stores';
	import * as api from '$lib/api.js';
	import InfiniteLoading from 'svelte-infinite-loading';

	let questions = [];
	let data = [];

	async function infiniteHandler({ detail: { loaded, complete } }) {
		let updated_at = '';
		if (questions.length) {
			updated_at = questions[questions.length - 1].updated_at;
		}
		let response = await api.post(`questions/`, { updated_at });

		if (response.status !== 200) {
			complete();
			return;
		}

		response = JSON.parse(await response.text());

		if (response.data.questions) {
			questions = response.data.questions;
		}
		if (questions.length) {
			for (let i = 0; i < questions.length; i++) {
				// questions[i]["tags"] = questions[i]["tags"].slice(1, -1);
				questions[i]['tags'] = questions[i]['tags'].split(',');
				// questions[i]["tid"] = questions[i]["tid"].slice(1, -1);
				questions[i]['tid'] = questions[i]['tid'].split(',');
				let asked_ts = Date.parse(questions[i].created_at);
				let updated_ts = Date.parse(questions[i].updated_at);
				let now = new Date();

				if (asked_ts == updated_ts) {
					let shown_ts = Math.floor((now - asked_ts) / 1000);
					if (shown_ts >= 259200) {
						asked_ts = new Date(questions[i].created_at);
						let year = asked_ts.getYear() + 1900;
						let month = asked_ts.getMonth() + 1;
						shown_ts = 'asked on ' + asked_ts.getDate() + '/' + month + '/' + year;
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
					questions[i].shown_ts = shown_ts;
				} else {
					let shown_ts = Math.floor((now - updated_ts) / 1000);
					if (shown_ts >= 259200) {
						asked_ts = new Date(questions[i].created_at);
						let year = updated_ts.getYear() + 1900;
						let month = updated_ts.getMonth() + 1;
						shown_ts = 'modified on ' + updated_ts.getDay() + '/' + month + '/' + year;
					} else if (172800 <= shown_ts && shown_ts < 259200) {
						shown_ts = 'modiffed 2 days ago';
					} else if (86400 <= shown_ts && shown_ts < 172800) {
						shown_ts = 'modified yesterday';
					} else if (3600 <= shown_ts && shown_ts < 8640000) {
						shown_ts = 'modified ' + Math.floor(shown_ts / 3600) + 'h ago';
					} else if (60 <= shown_ts && shown_ts < 3600) {
						shown_ts = 'modified ' + Math.floor(shown_ts / 60) + 'm ago';
					} else {
						shown_ts = 'modified ' + shown_ts + 's ago';
					}
					questions[i].shown_ts = shown_ts;
				}
			}
			data = [...data, ...questions];
			loaded();
		} else {
			complete();
		}
	}
</script>

<svelte:head>
	<title>All questions ❤ Kunjika</title>
</svelte:head>
<div class="row">
	<div class="col s12 m8 offset-m4 xl10 offset-xl2">
		<h1 class="text-xl font-bold">
			All questions
			{#if $page.data.user}
				<a href="/ask" class="btn" style="float:right;margin-right:50px;">Ask</a>
			{/if}
		</h1>
		<div class="row">
			{#each data as { id, slug, title, tags, shown_ts, uid, username, answers, views }}
				<hr
					style="border-bottom:1px solid;color:#eee;display:block;min-width:100%;margin-top:20px;margin-bottom:20px"
				/>
				<div style="margin-right:10px;flex-basis: 5%;max-width:5%;height:60px;float:left">
					<p style="text-align:center;font-size:16px;margin-top:5px">
						{answers}
					</p>
					<p style="text-align:center;font-size:10px;margin-top:10px;float:left">answers</p>
				</div>
				<div style="margin-right:10px;flex-basis: 5%;max-width:5%;height:60px;float:left">
					<p style="text-align:center;font-size:16px;margin-top:5px">
						{views}
					</p>
					<p style="text-align:center;font-size:10px;margin-top:10px;">views</p>
				</div>
				<div style="width:85%;float:left;position:relative">
					<a
						href="/questions/{id}/{slug}"
						style="text-decoration:none; color: #4285F4; font-size:16px; font-weight:400">{title}</a
					>
					<div style="margin-top:10px;clear:both" />
					{#each tags as tag, i}
						<a
							href="/questions/tagged/{tag}"
							class="btn"
							style="text-decoration:none; color: #fff;background-color: #4285F4;margin-right:10px">{tag}</a
						>
					{/each}
					<span style="float:right"
						>{shown_ts}
						<a href="/user/{uid}/{username}" style="text-decoration:none; color:#4285F4;"
							>{username}</a
						></span
					>
				</div>
				<div style="clear:both" />
			{/each}
			<svelte:component this={InfiniteLoading} on:infinite={infiniteHandler} />
			<hr style="border-bottom:1px solid;color:#eee;display:block;min-width:100%;margin-top:20px" />
		</div>
	</div>
</div>
