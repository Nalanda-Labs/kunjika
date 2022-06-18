<script context="module">
</script>

<script>
    import { goto } from "$app/navigation";
    import { session } from "$app/stores";
    import { onMount } from "svelte";
    import * as api from "$lib/api.js";
    import { post } from "$lib/utils.js";

    export let username;
    export let id;
    let name = "";
    let title = "";
    let designation = "";
    let location = "";
    let image_url = "";
    let image_alt = "";
    let git = "";
    let website = "";
    let twitter = "";
    let response = {};
    let karma = "";

    async function changeUsername() {
        let username_elem = document.getElementById("username");
        let username1 = username_elem.innerHTML;
        response = await api.get(
            `profile/${id}/username/${username1}`,
            $session.user.xsrf_token
        );
        if (response.code !== 200) {
            M.toast({ html: response.msg });
        } else if (response.data) {
            await post("/logout");
            $session.user = null;
            goto("/login");
        } else {
            M.toast({ html: "Username taken!" });
        }
    }
    async function closeModal() {
        const elem = document.getElementById("modal1");
        M.Modal.init(elem);
        const m = M.Modal.getInstance(elem);
        m.destroy();
    }

    onMount(async () => {
        response = await api.get(`user/${id}/${username}`);

        if (response.code == 200) {
            response = response.data;
            name = response.name;
            title = response.title;
            designation = response.designation;
            location = response.location;
            username = response.username;
            git = response.git;
            website = response.website;
            twitter = response.twitter;
            karma = response.karma;

            if (response.image_url !== "") {
                image_url = response.image_url;
                image_alt = name;
            } else {
                image_alt = "No profile photo";
            }
            if (title === "" && $session.user === response.username) {
                title = "Click to edit your title";
            }
            if (name === "" && $session.user === response.username) {
                name = "Click to edit your name";
            }
            if (designation === "" && $session.user === response.username) {
                designation = "Click to edit your designation";
            }
            if (location === "" && $session.user === response.username) {
                location = "Click to edit your location";
            }
            if (git === "" && $session.user === response.username) {
                git = "Add your git url";
            }
            if (website === "" && $session.user === response.username) {
                website = "Add your website link";
            }
            if (twitter === "" && $session.user === response.username) {
                twitter = "Add your twitter profile";
            }
        }

        let username_elem = document.getElementById("username");

        username_elem.addEventListener(
            "blur",
            async function () {
                if (username != username_elem.innerHTML) {
                    const elem = document.getElementById("modal1");
                    M.Modal.init(elem);
                    const m = M.Modal.getInstance(elem);
                    m.open();
                }
            },
            false
        );

        let title_elem = document.getElementById("title");

        title_elem.addEventListener(
            "blur",
            async function () {
                if (title != title_elem.innerHTML) {
                    let title1 = title_elem.innerHTML;

                    response = await api.get(
                        `profile/${id}/title/${title1}/`,
                        $session.user.xsrf_token
                    );
                    if (response.code !== 200) {
                        M.toast({ html: response.msg });
                    } else {
                        title = title_elem.innerHTML;
                    }
                }
            },
            false
        );

        let name_elem = document.getElementById("name");

        name_elem.addEventListener(
            "blur",
            async function () {
                if (name != name_elem.innerHTML) {
                    let name1 = name_elem.innerHTML;

                    response = await api.get(
                        `profile/${id}/name/${name1}/`,
                        $session.user.xsrf_token
                    );
                    if (response.code !== 200) {
                        M.toast({ html: response.msg });
                    } else {
                        name = name_elem.innerHTML;
                    }
                }
            },
            false
        );

        let d_elem = document.getElementById("designation");

        d_elem.addEventListener(
            "blur",
            async function () {
                if (designation != d_elem.innerHTML) {
                    let d1 = d_elem.innerHTML;

                    response = await api.get(
                        `profile/${id}/designation/${d1}/`,
                        $session.user.xsrf_token
                    );
                    if (response.code !== 200) {
                        M.toast({ html: response.msg });
                    } else {
                        designation = d_elem.innerHTML;
                    }
                }
            },
            false
        );

        let location_elem = document.getElementById("location");

        location_elem.addEventListener(
            "blur",
            async function () {
                if (location != location_elem.innerHTML) {
                    let location1 = location_elem.innerHTML;

                    response = await api.get(
                        `profile/${id}/location/${location1}/`,
                        $session.user.xsrf_token
                    );
                    if (response.code !== 200) {
                        M.toast({ html: response.msg });
                    } else {
                        location = location_elem.innerHTML;
                    }
                }
            },
            false
        );
    });
</script>

<div class="row" style="margin-top:10px">
    {#if $session.user && $session.user.username == username}
        <a class="btn" href="/edit-profile" style="float: right;"
            >Edit Profile</a
        >
    {/if}
    <div class="col-12 col-sm-12 col-md-2" style="float:left;margin-right:10px">
        <img
            src="{image_url}?s=160"
            alt="{username}'s proile image"
            width="160px"
        />
        <p>
            <a href="https://en.gravatar.com/site/signup/">Update gravatar</a> to
            change it.
        </p>
        <span style="text-align: center"
            ><span style="font-size:24px">{karma}</span><span
                style="position:relative;top:-5px">&nbsp;Karma</span
            ></span
        >
    </div>
    <div class="col-12 col-sm-12 col-md-5" style="float:left">
        <table style="display: inline;">
            <tr>
                {#if $session.user && $session.user.username == username}
                    <td
                        ><span
                            contenteditable="true"
                            id="username"
                            style="font-size:20px;font-weight: 500;"
                            >{username}</span
                        ></td
                    >
                {:else}
                    <td><span id="username">{username}</span></td>
                {/if}
            </tr>
            <tr>
                {#if $session.user && $session.user.username == username}
                    <td
                        ><span
                            contenteditable="true"
                            id="title"
                            data-placeholder="title; click to edit"
                            >{title}</span
                        ></td
                    >
                {:else}
                    <td><span id="title">{title}</span></td>
                {/if}
            </tr>
            <tr>
                {#if $session.user && $session.user.username == username}
                    <td
                        ><span
                            contenteditable="true"
                            id="name"
                            data-placeholder="name; click  edit">{name}</span
                        ></td
                    >
                {:else}
                    <td><span id="name">{name}</span></td>
                {/if}
            </tr>
            <tr>
                {#if $session.user && $session.user.username == username}
                    <td
                        ><span
                            contenteditable="true"
                            id="designation"
                            data-placeholder="designation; click to edit"
                            >{designation}</span
                        ></td
                    >
                {:else}
                    <td><span id="designation">{designation}</span></td>
                {/if}
            </tr>
        </table>
    </div>
    <div class="col12 sm12 md4" style="float:left;margin-left:200px">
        <table>
            <tr>
                {#if $session.user && $session.user.username == username}
                    <td>
                        <i class="fas fa-map-marker-alt" style="color:#666" />
                        <span
                            contenteditable="true"
                            id="location"
                            style="display: inline-block;min-width: 100px;"
                            data-placeholder="location; click to edit"
                        >
                            {location}
                        </span>
                    </td>
                {:else}
                    <td>
                        <i class="fas fa-map-marker-alt" style="color:#666" />
                        <span id="location">
                            {location}
                        </span>
                    </td>
                {/if}
            </tr>
            <tr>
                {#if $session.user && $session.user.username == username}
                    <td>
                        <i class="fab fa-github" style="color:#666" />
                        <a
                            class="anchor"
                            id="git"
                            href={git}
                            style="display: inline-block;min-width: 100px;"
                        >
                            {git}
                        </a>
                    </td>
                {:else}
                    <td>
                        <i class="fab fa-github" style="color:#666" />
                        <a class="anchor" id="git" href={git}>
                            {git}
                        </a>
                    </td>
                {/if}
            </tr>
            <tr>
                {#if $session.user && $session.user.username == username}
                    <td>
                        <i class="fas fa-link" style="color:#666" />
                        <a
                            class="anchor"
                            id="website"
                            href={website}
                            style="display: inline-block;min-width: 100px;"
                        >
                            {website}
                        </a>
                    </td>
                {:else}
                    <td>
                        <i class="fas fa-link" style="color:#666" />
                        <a class="anchor" id="website" href={website}>
                            {website}
                        </a>
                    </td>
                {/if}
            </tr>
            <tr>
                {#if $session.user && $session.user.username == username}
                    <td>
                        <i class="fab fa-twitter" style="color:#666" />
                        <a
                            class="anchor"
                            id="twitter"
                            href={twitter}
                            style="display: inline-block;min-width: 100px;"
                        >
                            {twitter}
                        </a>
                    </td>
                {:else}
                    <td>
                        <i class="fab fa-twitter" style="color:#666" />
                        <a id="twitter" class="anchor" href={twitter}>
                            {twitter}
                        </a>
                    </td>
                {/if}
            </tr>
        </table>
    </div>
    <div class="modal" id="username-change">
        <div class="modal-content">
            <h4>Username Change</h4>
            <p>Changing Username Will log you out.</p>
        </div>
        <div class="modal-footer">
            <a href="#!" class="modal-close waves-effect waves-green btn-flat"
                >Agree</a
            >
        </div>
    </div>
    <div id="modal1" class="modal" style="max-width: 400px;">
        <div class="modal-content">
            <h4>Changing Username</h4>
            <p>You will be logged out for a username change!</p>
        </div>
        <div class="modal-footer">
            <a
                href="#!"
                class="modal-close waves-effect waves-green btn-flat"
                id="ok"
                on:click={changeUsername}>OK</a
            >
            <a
                href="#!"
                class="modal-close waves-effect waves-green btn-flat"
                id="cancel"
                on:click={closeModal}>Cancel</a
            >
        </div>
    </div>
</div>

<style>
    span[contenteditable] {
        display: inline-block;
    }
    span[contenteditable]:empty::before {
        content: attr(data-placeholder);
        display: inline-block;
    }
    span[contenteditable]:empty:focus::before {
        content: "Start typing";
    }
</style>
