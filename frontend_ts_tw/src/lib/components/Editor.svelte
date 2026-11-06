<script lang="ts">
    import { tick } from 'svelte';
    import * as api from '#lib/api.ts';
    import * as cmd from '#lib/editor.ts';
    import { parseMarkdown } from '#lib/markdown.ts';
    import Icon, { type IconName } from './Icon.svelte';
    import Markdown from './Markdown.svelte';
    import Modal from './Modal.svelte';

    let {
        value = $bindable(''),
        minlength = 20,
        maxlength = 100000,
        preview = true
    }: { value?: string; minlength?: number; maxlength?: number; preview?: boolean } = $props();

    const MAX_IMAGE_BYTES = 2 * 1024 * 1024;

    let textarea: HTMLTextAreaElement;
    let uploadOpen = $state(false);
    let uploadError = $state('');
    let uploading = $state(false);
    let fileInput = $state<HTMLInputElement>();

    let rendered = $derived(preview ? parseMarkdown(value) : '');

    async function apply(command: cmd.Command) {
        const r = command(value, textarea.selectionStart, textarea.selectionEnd);
        value = r.value;
        await tick();
        textarea.focus();
        textarea.setSelectionRange(r.start, r.end);
    }

    function onKeydown(e: KeyboardEvent) {
        const command = cmd.shortcutFor(e);
        if (command) {
            e.preventDefault();
            apply(command);
        }
    }

    const tools: { icon: IconName; label: string; command: cmd.Command }[] = [
        { icon: 'bold', label: 'Bold [Ctrl/Cmd + B]', command: cmd.bold },
        { icon: 'italic', label: 'Italics [Ctrl/Cmd + I]', command: cmd.italic },
        { icon: 'link', label: 'Add link [Ctrl/Cmd + K]', command: cmd.link },
        { icon: 'list', label: 'Unordered list', command: cmd.unorderedList },
        { icon: 'list-ordered', label: 'Ordered list', command: cmd.orderedList }
    ];
    const blocks: { icon: IconName; label: string; command: cmd.Command }[] = [
        { icon: 'code', label: 'Code block', command: cmd.codeBlock },
        { icon: 'note', label: 'Note', command: cmd.note },
        { icon: 'info', label: 'Tip', command: cmd.tip },
        { icon: 'warning', label: 'Warning', command: cmd.warning }
    ];

    function openUpload() {
        uploadError = '';
        uploadOpen = true;
    }

    async function upload(e: SubmitEvent) {
        e.preventDefault();
        const file = fileInput?.files?.[0];
        if (!file) {
            uploadError = 'No image selected!';
            return;
        }
        if (file.size > MAX_IMAGE_BYTES) {
            uploadError = 'Max file size is 2MB';
            return;
        }
        const form = new FormData();
        form.append('file', file);
        uploading = true;
        const res = await api.upload('image-upload', form);
        const j = await api.json<{ url?: string; message?: string }>(res);
        uploading = false;
        if (res.status === 200 && j.url) {
            uploadOpen = false;
            await apply(cmd.insertText(`![alt](${j.url})`));
        } else {
            uploadError = j.message || 'Upload failed, try again.';
        }
    }

    const toolBtn =
        'group relative inline-flex h-8 w-8 items-center justify-center rounded text-gray-600 hover:bg-gray-100 hover:text-gray-900';
</script>

<div class="rounded-md border border-gray-300">
    <div class="flex flex-wrap items-center gap-0.5 border-b border-gray-200 bg-gray-50 px-1 py-1" role="toolbar">
        {#each tools as t (t.label)}
            <button type="button" class={toolBtn} title={t.label} aria-label={t.label} onclick={() => apply(t.command)}>
                <Icon name={t.icon} size={16} />
            </button>
        {/each}
        <button type="button" class={toolBtn} title="Add image" aria-label="Add image" onclick={openUpload}>
            <Icon name="image" size={16} />
        </button>
        {#each blocks as t (t.label)}
            <button type="button" class={toolBtn} title={t.label} aria-label={t.label} onclick={() => apply(t.command)}>
                <Icon name={t.icon} size={16} />
            </button>
        {/each}
        <a class={toolBtn} href="/markdown-help" target="_blank" title="Markdown help" aria-label="Markdown help">
            <Icon name="help" size={16} />
        </a>
    </div>
    <textarea
        bind:this={textarea}
        bind:value
        onkeydown={onKeydown}
        name="content"
        id="textAreaContent"
        rows="10"
        {minlength}
        {maxlength}
        required
        placeholder="Write your content here (markdown supported)..."
        class="block min-h-72 w-full resize-y rounded-b-md border-0 px-3 py-2 font-mono text-sm focus:ring-0 focus:outline-none"
    ></textarea>
</div>

{#if preview && value}
    <div class="mt-3 rounded-md border border-dashed border-gray-300 p-3">
        <Markdown html={rendered} />
    </div>
{/if}

<Modal bind:open={uploadOpen} title="Image Upload">
    <form onsubmit={upload} class="space-y-3">
        <p class="text-sm text-gray-600">Upload an image (max 2MB).</p>
        {#if uploadError}<div class="alert-danger">{uploadError}</div>{/if}
        <input bind:this={fileInput} type="file" name="image" accept="image/*" id="image" class="block text-sm" />
        <div class="flex justify-end gap-2">
            <button type="button" class="btn-secondary" onclick={() => (uploadOpen = false)}>Close</button>
            <button type="submit" class="btn-primary" disabled={uploading}>Upload</button>
        </div>
    </form>
</Modal>
