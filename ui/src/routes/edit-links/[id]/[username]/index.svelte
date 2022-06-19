<script context="module">
    export function preload({ params }, { user }) {
        if (!user) {
            this.redirect(302, `/login`);
        }
    }
</script>

<script>
    import { goto } from "$app/navigation";
    import { session, page } from "$app/stores";
    import { onMount } from "svelte";
    import { post, get } from "$lib/api.js";

    let uid = $session.user.id;
    let uid1 = $page.params.id;
    let username = $page.params.username;
    let website = "";
    let twitter = "";
    let git = "";

    onMount(async () => {
        // do not math types as in response sometimes uid is integer and sometimes string
        // TODO: Fix this
        if (uid != uid1) {
            goto(`/user/${uid}/${username}`);
        }
        if ($session.user) {
            const response = await get(
                `edit-links/${uid}`,
                $session.user.xsrf_token
            );
            if (response.code === 200) {
                website = response.data.website;
                git = response.data.git;
                twitter = response.data.twitter;
            } else {
                M.toast({ html: response.msg });
            }
        } else {
            goto("/login");
        }
    });
    async function submit() {
        const response = await post(
            `edit-links/${uid}`,
            { git, twitter, website },
            $session.user.xsrf_token
        );
        if (response.code === 200) {
            goto(`/user/${uid}/${username}`);
        } else {
            M.toast({ html: response.msg });
        }
    }
</script>

<div>
    <h3>Edit your social links</h3>
    <hr />
    <form
        on:submit|preventDefault={submit}
        class="col s6"
        style="max-width: 600px;"
    >
        <div class="input-field col s12 m6">
            <input bind:value={website} id="website" type="text" />
            <label for="website">Website</label>
        </div>
        <div class="input-field col s12 m6">
            <input bind:value={git} id="git" type="text" />
            <label for="git">Git</label>
        </div>
        <div class="input-field col s12 m6">
            <input bind:value={twitter} type="text" id="twitter" />
            <label for="twitter">Twitter</label>
        </div>
        <div class="b-wrapper">
            <button class="btn" type="submit" name="action"
                >Submit
                <i class="material-icons right">send</i>
            </button>
        </div>
    </form>
</div>
