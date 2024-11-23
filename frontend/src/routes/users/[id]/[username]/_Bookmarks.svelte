<script>
	import * as api from '$lib/api.js';
	import { onMount } from 'svelte';
	import { browser } from '$app/environment';
	import getCookie from '../../../../lib/cookie.js';

	export let id;
	let data = [];
	let bookmarks = [];
	let uat = '';
	let count = 0;
	let pages = 0; // total no. of pages
	let page = 1; // current page
	let bookmarks_per_page = 30;

	function processBookmarks(data) {
		bookmarks = data;
		for (let i = 0; i < bookmarks.length; i++) {
			bookmarks[i]['tags'] = bookmarks[i]['tags'].split(',');
			bookmarks[i]['tid'] = bookmarks[i]['tid'].split(',');
			let asked_ts = new Date(bookmarks[i].cat * 1000);
			let updated_ts = new Date(bookmarks[i].uat * 1000);
			let now = new Date();
			bookmarks[i].updated_ts = updated_ts;

			if (asked_ts == updated_ts) {
				let shown_ts = Math.floor((now - asked_ts) / 1000);

				if (shown_ts >= 259200) {
					asked_ts = new Date(bookmarks[i].cat);
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
				bookmarks[i].shown_ts = shown_ts;
			} else {
				let shown_ts = Math.floor((now - updated_ts) / 1000);
				if (shown_ts >= 259200) {
					asked_ts = new Date(bookmarks[i].cat);
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
				bookmarks[i].shown_ts = shown_ts;
			}
		}
	}

	async function handleError(response) {
		response = JSON.parse(await response.text());
		if (
			response.status === false &&
			response.message == 'no rows returned by a query that expected to return at least one row'
		) {
			count = 0;
			bookmarks = [];
		} else {
			alert(response.message);
		}
	}

	onMount(async () => {
		if (browser) {
			let xsrf_token = getCookie('xsrf_token');
			let response = await api.post(
				`${id}/get-bookmarks-by-user/`,
				{ uat, bookmarks_per_page },
				xsrf_token
			);

			if (response.status === 200) {
				response = JSON.parse(await response.text());

				let data = response.data.map((t) => t);
				count = response.count;
				pages = Math.floor(count / bookmarks_per_page);
				if (count % bookmarks_per_page !== 0) {
					pages += 1;
				}

				processBookmarks(data);
			} else {
				handleError(response);
			}
		}
	});

	async function nextPage() {
		page += 1;
		uat = bookmarks[bookmarks.length - 1].updated_ts;
		if (browser) {
			let xsrf_token = getCookie('xsrf_token');
			let response = await api.post(
				`${id}/get-bookmarks-by-user/`,
				{ uat, bookmarks_per_page },
				xsrf_token
			);

			if (response.status === 200) {
				response = JSON.parse(await response.text());

				let data = response.data.map((t) => t);
				count = response.count;
				pages = Math.floor(count / bookmarks_per_page);
				if (count % bookmarks_per_page !== 0) {
					pages += 1;
				}

				processBookmarks(data);
			} else {
				alert(response.message);
			}
		}
	}

	async function prevPage() {
		page -= 1;
		tag = bookmarks[0].updated_ts;
		if (browser) {
			let xsrf_token = getCookie('xsrf_token');
			let response = await api.post(
				`${id}/get-bookmarks-by-user/`,
				{
					uat,
					bookmarks_per_page,
					direction: 'back'
				},
				xsrf_token
			);

			if (response.status === 200) {
				response = JSON.parse(await response.text());

				let data = response.data.map((t) => t);
				count = response.count;
				pages = Math.floor(count / bookmarks_per_page);
				if (count % bookmarks_per_page !== 0) {
					pages += 1;
				}

				processBookmarks(data);
			} else {
				handleError(response);
			}
		}
	}

	async function firstPage() {
		page = 1;
		uat = '';
		if (browser) {
			let xsrf_token = getCookie('xsrf_token');
			let response = await api.post(
				`${id}/get-bookmarks-by-user/`,
				{ uat, bookmarks_per_page },
				xsrf_token
			);

			if (response.status === 200) {
				response = JSON.parse(await response.text());

				let data = response.data.map((t) => t);
				count = response.count;
				pages = Math.floor(count / bookmarks_per_page);
				if (count % bookmarks_per_page !== 0) {
					pages += 1;
				}

				processBookmarks(data);
			} else {
				handleError(response);
			}
		}
	}
</script>

<div style="margin:20px">
	<div class="row">
		{#each bookmarks as { answer_id, question_id, slug, title, tags, shown_ts }}
			<hr
				style="border-bottom:1px solid;color:#ccc;display:block;min-width:100%;margin-top:20px;margin-bottom:20px"
			/>
			<div style="width:85%;float:left;position:relative">
				{#if answer_id != 0}
					<a
						href="/questions/{question_id}/{slug}#{answer_id}"
						class="blue-text text-darken-2"
						style="text-decoration:none; font-size:16px; font-weight:400">{title}</a
					>
				{:else}
					<a
						href="/questions/{question_id}/{slug}"
						class="blue-text text-darken-2"
						style="text-decoration:none; font-size:16px; font-weight:400">{title}</a
					>
				{/if}
				<div style="margin-top:20px;clear:both"></div>
				{#each tags as tag, i}
					<a
						href="/questions/tagged/{tag}"
						class="light-blue darken-2"
						style="display:inline;padding:5px;border-radius:3px;text-decoration:none; background-color:#f0f0ff;margin-right:10px;font-size:12px"
						>{tag}</a
					>
				{/each}
				<span style="float:right">{shown_ts}</span>
			</div>
			<div style="clear:both"></div>
		{/each}
		<hr style="border-bottom:1px solid;color:#ccc;display:block;min-width:100%;margin-top:20px" />
	</div>
	<div style="clear:both;margin:auto;width:100%;margin-top:20px"></div>
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
