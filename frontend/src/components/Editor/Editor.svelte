<script>
	import {
		addBoldCommand,
		addCodeBlockCommand,
		addItalicCommand,
		addLinkCommand,
		addNoteCommand,
		addOrderedListCommand,
		addTipCommand,
		addUnorderedListCommand,
		addWarningCommand,
		parseMarkdown,
		useKeyCombinations,
		openForm
	} from '../../lib/utils/editor/utils.editor';
	import hljs from 'highlight.js';

	let contentTextArea;
	export let contentValue;
	export let markup;
	export let minlength;
	export let maxlength;

	const handlePreview = async () => {
		const bodyEditor = {
			content: contentValue
		};

		markup = parseMarkdown(bodyEditor.content);
	};
</script>

<div class="editor-icons">
	<div class="basic">
		<!-- svelte-ignore a11y-click-events-have-key-events -->
		<!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
		<p
			on:click={() => {
				addBoldCommand(contentTextArea);
			}}
			class="tooltip"
		>
			<i class="fa-solid fa-bold" />
			<span class="tooltiptext">Bold command [Cmd/Ctrl(Shift) + B]</span>
		</p>
		<!-- svelte-ignore a11y-click-events-have-key-events -->
		<!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
		<p
			class="tooltip"
			on:click={() => {
				addItalicCommand(contentTextArea);
			}}
		>
			<i class="fa-solid fa-italic" />
			<span class="tooltiptext"> Italics command [Cmd/Ctrl(Shift) + I] </span>
		</p>
		<!-- svelte-ignore a11y-click-events-have-key-events -->
		<!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
		<p
			class="tooltip"
			on:click={() => {
				addLinkCommand(contentTextArea);
			}}
		>
			<i class="fa-solid fa-link" />
			<span class="tooltiptext">Add link command [Cmd/Ctrl(Shift) + K]</span>
		</p>
		<!-- svelte-ignore a11y-click-events-have-key-events -->
		<!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
		<p
			class="tooltip"
			on:click={() => {
				addUnorderedListCommand(contentTextArea);
			}}
		>
			<i class="fa-solid fa-list" />
			<span class="tooltiptext">Add unordered list command</span>
		</p>
		<!-- svelte-ignore a11y-click-events-have-key-events -->
		<!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
		<p
			class="tooltip"
			on:click={() => {
				addOrderedListCommand(contentTextArea);
			}}
		>
			<i class="fa-solid fa-list-ol" />
			<span class="tooltiptext">Add ordered list command</span>
		</p>
		<!-- svelte-ignore a11y-click-events-have-key-events -->
		<!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
		<p
			class="tooltip"
			on:click={() => {
				// addImageCommand(contentTextArea);
				openForm();
			}}
		>
			<i class="fa-solid fa-image" />
			<span class="tooltiptext">Add image command</span>
		</p>
		<!-- svelte-ignore a11y-click-events-have-key-events -->
		<!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
		<p
			class="tooltip"
			on:click={() => {
				addCodeBlockCommand(contentTextArea);
			}}
		>
			<i class="fa-solid fa-code" />
			<span class="tooltiptext">Code block command</span>
		</p>
		<!-- svelte-ignore a11y-click-events-have-key-events -->
		<!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
		<p
			class="tooltip"
			on:click={() => {
				addNoteCommand(contentTextArea);
			}}
		>
			<i class="fa-solid fa-note-sticky" />
			<span class="tooltiptext">Note command</span>
		</p>
		<!-- svelte-ignore a11y-click-events-have-key-events -->
		<!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
		<p
			class="tooltip"
			on:click={() => {
				addTipCommand(contentTextArea);
			}}
		>
			<i class="fa-solid fa-circle-info" />
			<span class="tooltiptext">Tip command</span>
		</p>
		<!-- svelte-ignore a11y-click-events-have-key-events -->
		<!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
		<p
			class="tooltip"
			on:click={() => {
				addWarningCommand(contentTextArea);
			}}
		>
			<i class="fa-solid fa-warning" />
			<span class="tooltiptext">Warning command</span>
		</p>
	</div>
	<div class="others"></div>
</div>

<textarea
	bind:this={contentTextArea}
	bind:value={contentValue}
	on:focus={(event) => {
		if (event.target) {
			useKeyCombinations(event, contentTextArea);
		}
	}}
	on:input={() => {
		handlePreview();
		hljs.highlightAll();
	}}
	on:keyup={() => {
		handlePreview();
		hljs.highlightAll();
	}}
	name="content"
	class="form-control"
	rows="10"
	style="min-height: 300px"
	id="textAreaContent"
	{minlength}
	{maxlength}
	placeholder="Write your content here (markdown supported)..."
	data-input-field
	required
></textarea>
