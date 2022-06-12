<script context="module">
	import { get } from '$lib/api.js';

	export async function preload({ params }, { user }) {
		if (!user) {
			this.redirect(302, `/login`);
		}

		const { id } = params;
		const { question } = await get(`question/${id}`, null);
		return { question, id };
	}
</script>

<script>
	import Editor from './_Editor.svelte';

	export let id;
	export let question;
</script>

<Editor {question} {id}/>
