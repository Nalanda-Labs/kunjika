<script context="module">
    export function preload({ params }, { user }) {
        if (!user) {
            this.redirect(302, `/login`);
        }
    }
</script>

<script>
    import Editor from "./_editor.svelte";

    let question = { title: "", body: "", tagList: [] };
</script>

<svelte:head>
    <title>Create Topic ❤ Kunjika</title>
</svelte:head>

<Editor {question}/>
