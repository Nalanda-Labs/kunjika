<script lang="ts">
    import * as api from '#lib/api.ts';
    import type { ActionData } from './$types';

    let { form }: { form: ActionData } = $props();

    let email = $state('');
    let step = $state<1 | 2>(1);
    let error = $state('');
    let sending = $state(false);

    $effect(() => {
        // A failed code submission comes back with the email: stay on step 2.
        if (form?.email) {
            email = form.email;
            step = 2;
        }
    });

    async function requestCode(e: SubmitEvent) {
        e.preventDefault();
        error = '';
        sending = true;
        const res = await api.post('submit-email', { email });
        sending = false;
        if (res.status === 200) step = 2;
        else if (res.status === 401) error = 'You are unauthorized to login!';
        else error = 'Something went wrong! Try again or contact support!';
    }
</script>

<svelte:head>
    <title>Login</title>
</svelte:head>

<div class="mx-auto flex min-h-[70vh] max-w-sm flex-col justify-center">
    {#if step === 1}
        <h3 class="text-2xl font-normal">Login</h3>
        <p class="mt-2 mb-4"><a href="/register" class="link">Need an account?</a></p>
        {#if error}<p class="mb-3 text-danger">{error}</p>{/if}
        <form method="post" onsubmit={requestCode} class="space-y-4">
            <div>
                <label for="email" class="label">Email</label>
                <input type="email" id="email" name="email" class="input" required bind:value={email} />
            </div>
            <div class="flex justify-center">
                <button class="btn-primary" type="submit" disabled={sending}>Submit</button>
            </div>
        </form>
    {:else}
        <h3 class="text-2xl font-normal">Submit Sign-in Code</h3>
        <p class="mt-2 mb-4">An email has been sent with sign-in code.</p>
        {#if form?.errors}<p class="mb-3 text-danger">{form.errors}</p>{/if}
        <form method="post" action="?/login" class="space-y-4">
            <input type="hidden" name="email" value={email} />
            <div>
                <label for="sign-in-code" class="label">Sign-in Code</label>
                <input type="text" id="sign-in-code" name="sign_in_code" class="input" required autocomplete="one-time-code" />
            </div>
            <div class="flex justify-center">
                <button class="btn-primary" type="submit">Submit</button>
            </div>
        </form>
    {/if}
</div>
