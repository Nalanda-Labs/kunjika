<script context="module">
	export function preload({ params }, { user }) {}
</script>

<script>
	import { page } from '$app/stores';
	import * as api from '$lib/api.js';
	import { onMount } from 'svelte';

	let questions = [];
	let uat = '';
	let count = 0;
	let pages = 0; // total no. of pages
	let current_page = 1; // current page
	let questions_per_page = 30;

	function processQuestions(data) {
		questions = data;
		for (let i = 0; i < questions.length; i++) {
			questions[i]['tags'] = questions[i]['tags'].split(',');
			questions[i]['tid'] = questions[i]['tid'].split(',');
			let asked_ts = new Date(questions[i].cat * 1000);
			let updated_ts = new Date(questions[i].uat * 1000);
			let now = new Date();
			questions[i].updated_ts = updated_ts;

			if (asked_ts == updated_ts) {
				let shown_ts = Math.floor((now - asked_ts) / 1000);

				if (shown_ts >= 259200) {
					asked_ts = new Date(questions[i].cat);
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
					asked_ts = new Date(questions[i].cat);
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
	}

	async function handleError(response) {
		response = JSON.parse(await response.text());
		if (
			response.status === false &&
			response.message === 'no rows returned by a query that expected to return at least one row'
		) {
			count = 0;
			questions = [];
		} else {
			alert(response.message);
		}
	}

	onMount(async () => {
		let response = await api.post(`questions/tagged/${$page.params.tag}`, { uat, questions_per_page });

		if (response.status === 200) {
			response = JSON.parse(await response.text());

			let data = response.data.questions.map((t) => t);
			count = response.count;
			pages = Math.floor(count / questions_per_page);
			if (count % questions_per_page !== 0) {
				pages += 1;
			}

			processQuestions(data);
		} else {
			handleError(response);
		}
	});

	async function nextPage() {
		current_page += 1;
		uat = questions[questions.length - 1].updated_ts;
		let response = await api.post(`questions/tagged/${$page.params.tag}`, { uat, questions_per_page });

		if (response.status === 200) {
			response = JSON.parse(await response.text());

			let data = response.data.questions.map((t) => t);
			count = response.count;
			pages = Math.floor(count / questions_per_page);
			if (count % questions_per_page !== 0) {
				pages += 1;
			}

			processQuestions(data);
		} else {
			handleError(response);
		}
	}

	async function prevPage() {
		current_page -= 1;
		uat = questions[0].updated_ts;
		// this is because javascript sets the nanoseconds part to 0
		// however db stores the nanoseconds making backward direction
		// give wrong results. So we increase seconds by 1 which should
		// work in almost all cases.
		uat.setSeconds(uat.getSeconds() + 1);
		let response = await api.post(`questions/tagged/${$page.params.tag}`, { uat, questions_per_page, direction: 'back' });

		if (response.status === 200) {
			response = JSON.parse(await response.text());

			let data = response.data.questions.map((t) => t);
			count = response.count;
			pages = Math.floor(count / questions_per_page);
			if (count % questions_per_page !== 0) {
				pages += 1;
			}

			processQuestions(data);
		} else {
			response = JSON.parse(await response.text());
			if (
				response.status == 'fail' &&
				response.message == 'no rows returned by a query that expected to return at least one row'
			) {
				count = 0;
				questions = [];
			} else {
				handleError(response);
			}
		}
	}

	async function firstPage() {
		current_page = 1;
		uat = '';
		let response = await api.post(`questions/tagged/${$page.params.tag}`, { uat, questions_per_page });

		if (response.status === 200) {
			response = JSON.parse(await response.text());

			let data = response.data.questions.map((t) => t);
			count = response.count;
			pages = Math.floor(count / questions_per_page);
			if (count % questions_per_page !== 0) {
				pages += 1;
			}

			processQuestions(data);
		} else {
			alert(response.message);
		}
	}
</script>

<svelte:head>
	<title>All questions ❤ Kunjika</title>
</svelte:head>

<div style="margin-top:20px">
	<h4 class="text-xl font-bold">
		Questions tagged {$page.params.tag}({count} questions)
		{#if $page.data.user}
			<a href="/ask" style="float:right;margin-right:50px;text-decoration:none">Ask</a>
		{/if}
	</h4>
    <a class="btn btn-promary" href="/tags/info/{encodeURIComponent($page.params.tag)}" style="color:white">Info</a>
	<div class="row">
		{#each questions as { id, slug, title, tags, shown_ts, uid, username, answers, views, answer_accepted }}
			<hr
				style="border-bottom:1px solid;color:#ccc;display:block;min-width:100%;margin-top:20px;margin-bottom:20px"
			/>
			{#if answer_accepted}
				<div
					style="margin-right:0px;flex-basis: 5%;max-width:5%;height:60px;float:left;background-color: green;color: white;"
				>
					<p style="text-align:center;font-size:16px;margin-top:5px">
						{answers}
					</p>
					<p style="text-align:center;font-size:10px;margin-top:0px;float:left">answers</p>
				</div>
			{:else}
				<div style="margin-right:0px;flex-basis: 5%;max-width:5%;height:60px;float:left;">
					<p style="text-align:center;font-size:16px;margin-top:5px">
						{answers}
					</p>
					<p style="text-align:center;font-size:10px;margin-top:0px;float:left">answers</p>
				</div>
			{/if}
			<div style="margin-right:0px;flex-basis: 5%;max-width:5%;height:60px;float:left">
				<p style="text-align:center;font-size:16px;margin-top:5px">
					{views}
				</p>
				<p style="text-align:center;font-size:10px;margin-top:10px;">views</p>
			</div>
			<div style="width:85%;float:left;position:relative">
				<a
					href="/questions/{id}/{slug}"
					class="blue-text text-darken-2"
					style="text-decoration:none; font-size:16px; font-weight:400">{title}</a
				>
				<div style="margin-top:20px;clear:both"></div>
				{#each tags as tag, i}
					<a
						href="/questions/tagged/{encodeURIComponent(tag)}"
						class="light-blue darken-2"
						style="display:inline;padding:5px;border-radius:3px;text-decoration:none; background-color:#f0f0ff;margin-right:10px;font-size:12px"
						>{tag}</a
					>
				{/each}
				<span style="float:right"
					>{shown_ts}
					<a href="/users/{uid}/{username}" style="text-decoration:none; color:#4285F4;"
						>{username}</a
					></span
				>
			</div>
			<div style="clear:both"></div>
		{/each}
		<hr style="border-bottom:1px solid;color:#ccc;display:block;min-width:100%;margin-top:20px" />
	</div>
	<div style="clear:both;margin:auto;width:100%;margin-top:20px"></div>
	<div style="float: right;">
		<ul class="pagination">
			<!-- svelte-ignore a11y-invalid-attribute -->
			{#if current_page == 1}
				<li class="disabled"><i class="material-icons" title="first page">first_page</i></li>
				<li class="disabled">
					<i class="material-icons" title="previouse page">chevron_left</i>
				</li>
				{#if pages > 1}
					<li style="cursor:pointer" on:click={nextPage}>
						<i class="material-icons" title="next page">chevron_right</i>
					</li>
				{/if}
				{#if current_page != pages && pages != 0}
					<li class="disabled"><i class="material-icons" title="last page">last_page</i></li>
				{/if}
			{:else if current_page != pages}
				<li style="cursor:pointer" on:click={firstPage}>
					<i class="material-icons" title="first page">first_page</i>
				</li>
				<li style="cursor:pointer" on:click={prevPage}>
					<i class="material-icons" title="previouse page">chevron_left</i>
				</li>
				<li style="cursor:pointer" on:click={nextPage}>
					<i class="material-icons" title="next page">chevron_right</i>
				</li>
			{:else if current_page == pages}
				<li style="cursor:pointer" on:click={firstPage}>
					<i class="material-icons" title="first page">first_page</i>
				</li>
				<li style="cursor:pointer" on:click={prevPage}>
					<i class="material-icons" title="previouse page">chevron_left</i>
				</li>
				<li class="disabled"><i class="material-icons" title="next page">chevron_right</i></li>
				<li class="disabled"><i class="material-icons" title="last page">last_page</i></li>
			{/if}
		</ul>
	</div>
</div>
