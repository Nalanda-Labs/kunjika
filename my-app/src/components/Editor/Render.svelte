<script>
	import hljs from 'highlight.js';
	import '../../routes/highlight.css';

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

<section class="render" aria-label="feature">
	<div class="w-full">
		<div>
			<div class="w-full" bind:this={articleContainer}>
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