<script>
	import { page } from '$app/stores';
    import { goto } from '$app/navigation';
	import * as api from '$lib/api.js';
	import { onMount } from 'svelte';
	import Preview from '../../../../components/Editor/Preview.svelte';
	import { parseMarkdown } from '../../../../lib/utils/editor/utils.editor';

    let info = '';

    onMount(async () => {
		let response = await api.get(`get-tag-info/${encodeURIComponent($page.params.tag)}`);

		if (response.status === 200) {
			response = JSON.parse(await response.text());
			info = response.info;
            info = parseMarkdown(info);
        } else {
            response = JSON.parse(await response.text());
            alert(response.message);
        }
    });
</script>

<svelte:head>
	<title>Tag Info ❤ Kunjika</title>
</svelte:head>

<div style="margin-top:20px; max-width:800px;margin:auto">
    <h4 class="text-xl font-bold">About {$page.params.tag}</h4>
    <h5>Tag Info</h5>
	<hr />
    <Preview markup={info} />
    <button class="btn btn-primary" on:click="{() => goto(`/tags/edit/${encodeURIComponent($page.params.tag)}`)}">Update Info</button>
</div>