<script context="module">
    export function preload({ params }, { user }) {
        let [tag] = [params.tag];
        return { tag };
    }
</script>

<script>
    import { onMount } from "svelte";
    import * as api from "api.js";
    import "bytemd/dist/index.min.css";
    import "../../_utils.scss";
    import { stores, goto } from "@sapper/app";
    import Button, { Label } from "@smui/button";

    export let tag;
    export let slug;
    let value = "";

    let Viewer = null;

    const { session } = stores();

    onMount(async () => {
        const bytemd = await import("bytemd");
        Viewer = bytemd.Viewer;

        let response = await api.get(`tags/edit/${tag}`);

        if (response.info) {
            value = response.info;
        }
    });
    function onClick() {
        goto(`tags/edit/${tag}`);
    }
</script>

<svelte:head>
    <title>Info for tag {tag}</title>
</svelte:head>
<div class="topic" id="container">
    <h3>{tag} Info</h3>
    <hr />
    <div style="float:left; position:relative;width:calc(100% - 70px)">
        <svelte:component this={Viewer} {value} />
    </div>
    <div class="b-wrapper">
        <Button variant="raised" on:click={onClick}>
            <Label>Edit</Label>
        </Button>
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
