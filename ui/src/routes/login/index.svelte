<script context="module">
    export async function load({ session }) {
        if (session.user) {
            return {
                status: 302,
                redirect: "/questions",
            };
        }

        return {};
    }
</script>

<script>
    import * as api from "$lib/api.js";
    import ListErrors from "$lib/ListErrors.svelte";

    let email = "";
    let password = "";
    let errors = null;

    async function submit() {
        const response = await api.post(`login`, { email, password });

        if (response.success) {
            window.location.replace('/questions');
        } else {
            M.toast({html: 'Email or password not correct or you have not verified your email!'});
        }
    }
</script>

<svelte:head>
    <title>Login ❤ Kunjika</title>
</svelte:head>

<main>
    <div class="row">
        <div class="col-md-6 offset-md-3 col-xs-12">
            <h3 class="text-xs-center">Login</h3>
            <p class="text-xs-center">
                <a href="/register">Need an account?</a>
            </p>

            <ListErrors {errors} />

            <form on:submit|preventDefault={submit} class="col s6">
                <div class="row">
                    <div class="input-field col s12">
                        <input
                            class="validate"
                            type="email"
                            id="email"
                            required
                            bind:value={email}
                        />
                        <label for="email">Email</label>
                    </div>
                    <div class="input-field col s12">
                        <input
                            class="validate"
                            type="password"
                            id="password"
                            required
                            bind:value={password}
                            minlength="16"
                        />
                        <label for="password">Passsword</label>
                    </div>
                </div>
                <div class="b-wrapper">
                    <button class="btn" type="submit"
                        >Submit
                        <i class="material-icons right">send</i>
                    </button>
                </div>
            </form>
        </div>
    </div>
</main>
