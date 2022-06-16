<script context="module">
    // main navigation for the application
    import Activity from "./_Activity.svelte";
    import Profile from "./_Profile.svelte";

    // these are our 'pages' to navigate through (in order for this export to work the script tag MUST include context="module")
    export const navOptions = [
        { page: "Profile", component: Profile },
        { page: "Activity", component: Activity },
    ];
</script>
