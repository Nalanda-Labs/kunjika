<script>
    import "../routes/_utils.scss";
    export let taglist;
</script>

<ul style="list-style-type:none;padding:0px;display:inline">
    {#each taglist as tag}
        <li style="display:inline"><a href="/tags/topics/tagged/{tag}" style="text-decoration:none; color: #fff;background-color: #4285F4; padding:7px; margin-right:10px; border-radius: 4px">{tag}</a></li>
    {/each}
</ul>
