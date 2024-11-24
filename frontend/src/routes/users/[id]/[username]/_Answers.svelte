<script>
	import * as api from '$lib/api.js';
	import { afterUpdate, onMount } from 'svelte';
	import Edit from './_Edit.svelte';

	export let id;
	let data = [];
	let answers = [];
	let uat = '';
	let count = 0;
	let pages = 0; // total no. of pages
	let page = 1; // current page
	let answers_per_page = 30;

	function processQuestions(data) {
		answers = data;
		for (let i = 0; i < answers.length; i++) {
			answers[i]['tags'] = answers[i]['tags'].split(',');
			answers[i]['tid'] = answers[i]['tid'].split(',');
			let asked_ts = new Date(answers[i].cat * 1000);
			let updated_ts = new Date(answers[i].uat * 1000);
			let now = new Date();
			answers[i].updated_ts = updated_ts;

			if (asked_ts == updated_ts) {
				let shown_ts = Math.floor((now - asked_ts) / 1000);

				if (shown_ts >= 259200) {
					asked_ts = new Date(answers[i].cat);
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
				answers[i].shown_ts = shown_ts;
			} else {
				let shown_ts = Math.floor((now - updated_ts) / 1000);
				if (shown_ts >= 259200) {
					asked_ts = new Date(answers[i].cat);
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
				answers[i].shown_ts = shown_ts;
			}
		}
	}

	async function handleError(response) {
		response = JSON.parse(await response.text());
		if (
			response.status == 'fail' &&
			response.message == 'no rows returned by a query that expected to return at least one row'
		) {
			count = 0;
			questions = [];
		} else {
			alert(response.message);
		}
	}

	onMount(async () => {
		let response = await api.post(`${id}/get-answers-by-user/`, { uat });

		if (response.status === 200) {
			response = JSON.parse(await response.text());

			let data = response.data.map((t) => t);
			count = response.count;
			pages = Math.floor(count / answers_per_page);
			if (count % answers_per_page !== 0) {
				pages += 1;
			}

			processQuestions(data);
		} else {
			handleError(response);
		}
	});

	async function nextPage() {
		page += 1;
		uat = answers[answers.length - 1].updated_ts;
		let response = await api.post(`${id}/get-answers-by-user/`, { uat });

		if (response.status === 200) {
			response = JSON.parse(await response.text());

			let data = response.data.map((t) => t);
			count = response.count;
			pages = Math.floor(count / answers_per_page);
			if (count % answers_per_page !== 0) {
				pages += 1;
			}

			processQuestions(data);
		} else {
			alert(response.message);
		}
	}

	async function prevPage() {
		page -= 1;
		tag = answers[0].updated_ts;
		let response = await api.post(`${id}/get-answers-by-user/`, { uat, direction: 'back' });

		if (response.status === 200) {
			response = JSON.parse(await response.text());

			let data = response.data.map((t) => t);
			count = response.count;
			pages = Math.floor(count / answers_per_page);
			if (count % answers_per_page !== 0) {
				pages += 1;
			}

			processQuestions(data);
		} else {
			handleError(response);
		}
	}

	async function firstPage() {
		page = 1;
		uat = '';
		let response = await api.post(`${id}/get-answers-by-user/`, { uat });

		if (response.status === 200) {
			response = JSON.parse(await response.text());

			let data = response.data.map((t) => t);
			count = response.count;
			pages = Math.floor(count / answers_per_page);
			if (count % answers_per_page !== 0) {
				pages += 1;
			}

			processQuestions(data);
		} else {
			handleError(response);
		}
	}
</script>

<div style="margin:20px">
	<div class="row">
		{#each answers as { answer_id, question_id, slug, title, tags, shown_ts }}
			<hr
				style="border-bottom:1px solid;color:#ccc;display:block;min-width:100%;margin-top:20px;margin-bottom:20px"
			/>
			<div style="width:85%;float:left;position:relative">
				<a
					href="/questions/{question_id}/{slug}#{answer_id}"
					class="blue-text text-darken-2"
					style="text-decoration:none; font-size:16px; font-weight:400">{title}</a
				>
				<div style="margin-top:20px;clear:both" />
				{#each tags as tag, i}
					<a
						href="/questions/tagged/{encodeURIComponent(tag)}"
						class="light-blue darken-2"
						style="display:inline;padding:5px;border-radius:3px;text-decoration:none; background-color:#f0f0ff;margin-right:10px;font-size:12px"
						>{tag}</a
					>
				{/each}
				<span style="float:right">{shown_ts}</span>
			</div>
			<div style="clear:both" />
		{/each}
		<hr style="border-bottom:1px solid;color:#ccc;display:block;min-width:100%;margin-top:20px" />
	</div>
	<div style="clear:both;margin:auto;width:100%;margin-top:20px" />
	<div style="float: right;">
		<ul class="pagination">
			<!-- svelte-ignore a11y-invalid-attribute -->
			{#if page == 1}
				<li class="disabled"><i class="material-icons" title="first page">first_page</i></li>
				<li class="disabled">
					<i class="material-icons" title="previouse page">chevron_left</i>
				</li>
				{#if pages > 1}
					<li style="cursor:pointer" on:click={nextPage}>
						<i class="material-icons" title="next page">chevron_right</i>
					</li>
				{/if}
				{#if page != pages && pages != 0}
					<li class="disabled"><i class="material-icons" title="last page">last_page</i></li>
				{/if}
			{:else if page != pages}
				<li style="cursor:pointer" on:click={firstPage}>
					<i class="material-icons" title="first page">first_page</i>
				</li>
				<li style="cursor:pointer" on:click={prevPage}>
					<i class="material-icons" title="previouse page">chevron_left</i>
				</li>
				<li style="cursor:pointer" on:click={nextPage}>
					<i class="material-icons" title="next page">chevron_right</i>
				</li>
			{:else if page == pages}
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
