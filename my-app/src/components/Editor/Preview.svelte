<script>
	import hljs from 'highlight.js';
	import '../../routes/highlight.css';

	import { showPreview } from '$lib/stores/editor.store';
	import { onMount } from 'svelte';

	let articleContainer;

	onMount(() => {
		hljs.highlightAll();
		const blocks = articleContainer.querySelectorAll('pre code.hljs');
		Array.prototype.forEach.call(blocks, function (block) {
			const language = block.result.language;
			const small = document.createElement('small');
			small.classList.add('language', language);
			small.innerText = language;
			block.appendChild(small);
		});
	});

	export let markup;
</script>

<section class="section" aria-label="feature">
	<div class="w-full">
		<div class="preview full-text">
			<div class="main-text w-full border border-grey border-dashed" bind:this={articleContainer}>
				<p class="m-0.5">{@html markup}</p>
			</div>
		</div>
	</div>
</section>

<style>
	.preview.full-text {
		grid-template-columns: 0fr 5fr;
	}

	.side-text i {
		cursor: pointer;
	}
	@media (min-width: 992px) {
		.preview.full-text .side-text {
			position: fixed;
		}
	}
</style>