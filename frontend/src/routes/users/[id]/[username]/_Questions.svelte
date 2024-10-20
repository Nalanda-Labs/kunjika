<script>
	import * as api from '$lib/api.js';
	import { afterUpdate, onMount } from 'svelte';

	export let id;
	let questions = [];
	let cat = '';
	let limit = 20;

	onMount(async () => {
		const response = await api.get(`${id}/get-questions?limit=${limit}`, { cat });

		if (response.status === 200) {
			const text = await response.text();
			const j = text ? JSON.parse(text) : {};

			if (j.data) {
				// these questions are actually answers. the question has already been
				// fetched in the first request.
				questions = j.data.questions;
				for (var i = 0; i < questions.length; i++) {
					let asked_ts = new Date(questions[i].cat * 1000);
					let now = Date.now();
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
					questions[i].title = j.data.title;
					questions[i].taglist = j.data.tags.map((tag) => tag);
					questions[i].votes = j.data.votes;
					questions[i].answer_accepted = j.data.answer_accepted;
				}
			}
		}
	});
</script>
