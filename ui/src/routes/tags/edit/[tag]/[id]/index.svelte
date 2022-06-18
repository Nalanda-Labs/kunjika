<script context="module">
    export function preload({ params }, { user }) {
        if (!user) {
            this.redirect(302, `/login`);
        }
    }
</script>

<script>
    import Editor from "./_Editor.svelte";
    import { page } from "$app/stores";
    export let tag;
    export let id;

    let info = { info: "" };
    tag = $page.params.tag;
    id = $page.params.id;
</script>

<svelte:head>
    <title>Edit Tag • Arth</title>
</svelte:head>

<Editor {info} {tag} {id} />
