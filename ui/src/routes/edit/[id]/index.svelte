<script context="module">
    export function preload({ params }, { user }) {
        if (!user) {
            this.redirect(302, `/login`);
        }
    }
</script>

<script>
    import { page } from "$app/stores";
    import Editor from "./_Editor.svelte";

    export let id = $page.params.id;
    let topic = { title: "", body: "", taglist: [] };
</script>

<svelte:head>
    <title>Edit post ❤ Kunjika</title>
</svelte:head>

<Editor {topic} {id} />
