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
    let reputation = "";

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
            reputation = response.reputation;

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
                    let username1 = username_elem.innerHTML;
                    response = await api.post(
                        `profile/${id}/username/${username1}/`,
                        username1,
                        $session.user.xsrf_token
                    );
                    if (response.code !== 200) {
                        M.toast({ html: response.msg });
                    } else if (response.jwt) {
                        if (
                            confirm(
                                "You will be logged out for username change"
                            ) == true
                        ) {
                            await post("/logout");
                            $session.user = null;
                            goto("/login");
                        } else {
                            return;
                        }
                    }
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

                    response = await api.post(
                        `profile/${id}/title/${title1}/`,
                        title1.trim(),
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

                    response = await api.post(
                        `profile/${id}/name/${name1}/`,
                        name1.trim(),
                        $session.user.xsrf_token
                    );
                    if (response.code !== 200) {
                        M.toast({html: response.msg});
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

                    response = await api.post(
                        `profile/${id}/designation/${d1}/`,
                        d1.trim(),
                        $session.user.xsrf_token
                    );
                    if (response.code !== 200) {
                        M.toast({html: response.msg});
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

                    response = await api.post(
                        `profile/${id}/location/${d1}/`,
                        location1.trim(),
                        $session.user.xsrf_token
                    );
                    if (response.code !== 200) {
                        M.toast({html: response.msg});
                    } else {
                        location = location_elem.innerHTML;
                    }
                }
            },
            false
        );

        let git_elem = document.getElementById("git");

        git_elem.addEventListener(
            "blur",
            async function () {
                if (git != git_elem.innerHTML) {
                    let git1 = git_elem.innerHTML;

                    response = await api.post(
                        `profile/${id}/git/`,
                        { git: git1.trim() },
                        $session.user.xsrf_token
                    );
                    if (response.code !== 200) {
                        M.toast({html: response.msg});
                    } else {
                        git = git_elem.innerHTML;
                    }
                }
            },
            false
        );

        let website_elem = document.getElementById("website");

        website_elem.addEventListener(
            "blur",
            async function () {
                if (website != website_elem.innerHTML) {
                    let website1 = website_elem.innerHTML;

                    response = await api.post(
                        `profile/${id}/website/`,
                        { website: website1.trim() },
                        $session.user.xsrf_token
                    );
                    if (response.code !== 200) {
                        M.toast({html: response.msg});
                    } else {
                        website = website_elem.innerHTML;
                    }
                }
            },
            false
        );
        let twitter_elem = document.getElementById("twitter");

        twitter_elem.addEventListener(
            "blur",
            async function () {
                if (twitter != twitter_elem.innerHTML) {
                    let twitter1 = twitter_elem.innerHTML;

                    response = await api.post(
                        `profile/${id}/twitter/`,
                        { twitter: twitter1.trim() },
                        $session.user.xsrf_token
                    );
                    if (response.code !== 200) {
                        Swal.fire(response.msg);
                    } else {
                        twitter = twitter_elem.innerHTML;
                    }
                }
            },
            false
        );
    });
</script>

<div class="row" style="margin-top:10px">
    <div class="col-12 col-sm-12 col-md-2" style="float:left;margin-right:10px">
        <img
            src="{image_url}?s=160"
            alt="{username}'s proile image"
            width="160px"
        />
        <p>
            Your gravatar, <a href="https://en.gravatar.com/site/signup/"
                >update gravatar</a
            > to change it.
        </p>
        <span style="text-align: center"
            ><span style="font-size:24px">{reputation}</span><span
                style="position:relative;top:-5px">&nbsp;REPUTATION</span
            ></span
        >
    </div>
    <div class="col-12 col-sm-12 col-md-5" style="float:left">
        <table>
            <tr>
                {#if $session.user == username}
                    <td
                        ><span
                            contenteditable="true"
                            id="username"
                            style="font-size:20px;font-weight: 500;"
                            title="Click to edit">{username}</span
                        ></td
                    >
                {:else}
                    <td><span id="username">{username}</span></td>
                {/if}
            </tr>
            <tr>
                {#if $session.user == username}
                    <td
                        ><span
                            contenteditable="true"
                            id="title"
                            title="Click to edit">{title}</span
                        ></td
                    >
                {:else}
                    <td><span id="title">{title}</span></td>
                {/if}
            </tr>
            <tr>
                {#if $session.user == username}
                    <td><span contenteditable="true" id="name">{name}</span></td
                    >
                {:else}
                    <td><span id="name">{name}</span></td>
                {/if}
            </tr>
            <tr>
                {#if $session.user == username}
                    <td
                        ><span contenteditable="true" id="designation"
                            >{designation}</span
                        ></td
                    >
                {:else}
                    <td><span id="designation">{designation}</span></td>
                {/if}
            </tr>
        </table>
    </div>
    <div class="col-12 col-sm-12 col-md-4" style="display:inline">
        <table>
            <tr>
                {#if $session.user == username}
                    <td>
                        <i class="fas fa-map-marker-alt" style="color:#666" />
                        <span contenteditable="true" id="location">
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
                {#if $session.user == username}
                    <td>
                        <i class="fab fa-github" style="color:#666" />
                        <a
                            class="anchor"
                            contenteditable="true"
                            id="git"
                            href={git}
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
                {#if $session.user == username}
                    <td>
                        <i class="fas fa-link" style="color:#666" />
                        <a
                            class="anchor"
                            contenteditable="true"
                            id="website"
                            href={website}
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
                {#if $session.user == username}
                    <td>
                        <i class="fab fa-twitter" style="color:#666" />
                        <a
                            class="anchor"
                            contenteditable="true"
                            id="twitter"
                            href={twitter}
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
</div>
