<script>
	import { browser } from '$app/environment';
	import * as api from '$lib/api.js';
	import getCookie from '../../../../lib/cookie.js';
	import { goto } from '$app/navigation';
	import { afterUpdate, onMount } from 'svelte';

	export let id;
	export let username;
	let answers_count = 0;
	let questions_count = 0;
	let flags_count = 0;
	let votes_count = 0;
	let karma = 0;

	onMount(async () => {
		const response = await api.get(`${id}/summary`);

		if (response.status === 200) {
			const text = await response.text();
			const j = text ? JSON.parse(text) : {};

			answers_count = j.data.answers_count;
			questions_count = j.data.questions_count;
			karma = j.data.karma;
		}
	});
</script>

<div class="row justify-content-center align-items-center" style="margin-top:20px">
	<div class="row col-12">
		<div class="card" style="width: 18rem;margin: 10px;">
			<div class="card-body">
				<h5 class="card-title">Karma</h5>
				<p class="card-text">
					{karma} karma earned.
				</p>
			</div>
		</div>
		<div class="card" style="width: 18rem;margin: 10px;">
			<div class="card-body">
				<h5 class="card-title">Questions</h5>
				<p class="card-text">
					{questions_count} questions asked.
				</p>
			</div>
		</div>
		<div class="card" style="width: 18rem; margin: 10px;">
			<div class="card-body">
				<h5 class="card-title">Answers</h5>
				<p class="card-text">
					{answers_count} answers.
				</p>
			</div>
		</div>
		<div class="card" style="width: 18rem; margin: 10px;">
			<div class="card-body">
				<h5 class="card-title">Flags</h5>
				<p class="card-text">
					{flags_count} helpful flags.
				</p>
			</div>
		</div>
		<div class="card" style="width: 18rem; margin: 10px;">
			<div class="card-body">
				<h5 class="card-title">Votes</h5>
				<p class="card-text">
					{votes_count} votes cast.
				</p>
			</div>
		</div>
	</div>
</div>
