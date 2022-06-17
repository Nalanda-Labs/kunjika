<script context="module">
    export function preload({ params }, { user }) {
        if (!user) {
            this.redirect(302, `/login`);
        }
        let tag = params.tag;
        return { tag };
    }
</script>

<script>
    import Editor from "./_Editor.svelte";
    export let tag;
    let info = { info: "" };
</script>

<svelte:head>
    <title>Edit Tag • Arth</title>
</svelte:head>

<Editor {info} {tag} />

<style>
    @media (max-width: 720px) {
        .tags {
            width: 100%;
        }
    }
    @media (max-width: 4096px) {
        .tags {
            width: 800px;
        }
    }
</style>
