<script>
    import {goto} from "$app/navigation";
    import { onMount } from "svelte";
    import { session, page } from "$app/stores";
    import * as api from "$lib/api.js";
    import "bytemd/dist/index.min.css";

    let tag = $page.params.tag;
    let id = $page.params.id;
    let value = "";

    let Viewer = null;

    onMount(async () => {
        const bytemd = await import("bytemd");
        Viewer = bytemd.Viewer;

        let response = await api.get(`tags/edit/${tag}/${id}`, $session.user.xsrf_token);

        if (response.code === 200) {
            value = response.data;
        }
    });
    function onClick() {
        goto(`/tags/edit/${tag}/${id}`);
    }
</script>

<svelte:head>
    <title>Info for tag {tag} ❤ Kunjika</title>
</svelte:head>
<div class="topic" id="container">
    <h3>{tag} Info</h3>
    <hr />
    <div style="float:left; position:relative;width:calc(100% - 70px)">
        <svelte:component this={Viewer} {value} />
    </div>
    <div class="b-wrapper">
        <div class="b-wrapper">
            <!-- svelte-ignore a11y-invalid-attribute -->
            <button class="btn" on:click="{onClick}">Edit</button>
        </div>
    </div>
</div>

<style>
    @media (max-width: 720px) {
        .topic {
            width: 100%;
        }
    }
    @media (max-width: 4096px) {
        .topic {
            width: 800px;
        }
        :global(.bytemd-editor .CodeMirror) {
            height: 90% !important;
        }
    }
</style>
