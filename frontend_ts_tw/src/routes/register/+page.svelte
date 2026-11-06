<script lang="ts">
    import * as api from '#lib/api.ts';

    let username = $state('');
    let email = $state('');
    let helper = $state<{ text: string; ok: boolean } | null>(null);
    let message = $state<{ text: string; ok: boolean } | null>(null);
    let checkId = 0;

    async function checkAvailability() {
        if (username.length < 3 || username.length > 33) {
            helper = null;
            return;
        }
        const mine = ++checkId;
        const res = await api.post('check-username-availability', { username });
        const j = await api.json<{ message?: string }>(res);
        if (mine !== checkId) return;
        helper =
            j.message === 'username available'
                ? { text: 'Username available!', ok: true }
                : { text: 'Username unavailable!', ok: false };
    }

    async function submit(e: SubmitEvent) {
        e.preventDefault();
        if (username.length < 3 || username.length > 33) {
            helper = { text: 'Username must be at least three characters!', ok: false };
            return;
        }
        const res = await api.post('register', { username, email });
        const j = await api.json<{ message?: string }>(res);
        message =
            res.status === 200
                ? { text: 'An email verification email has been sent to you!', ok: true }
                : { text: j.message || 'Something went wrong, try again!', ok: false };
    }
</script>

<svelte:head>
    <title>Register ❤ Kunjika</title>
</svelte:head>

<div class="mx-auto flex min-h-[70vh] max-w-sm flex-col justify-center">
    <div id="registrationMessage">
        {#if message}
            <div class="{message.ok ? 'alert-success' : 'alert-danger'} mb-4 flex justify-between gap-2" role="alert">
                <span>{message.text}</span>
                <button type="button" aria-label="Close" onclick={() => (message = null)}>×</button>
            </div>
        {/if}
    </div>
    <h3 class="text-2xl font-normal">Register</h3>
    <p class="mt-2 mb-4"><a href="/login" class="link">Have an account?</a></p>
    <form onsubmit={submit} class="space-y-4">
        <div>
            <label for="username" class="label">Username</label>
            <input
                class="input"
                type="text"
                required
                bind:value={username}
                onkeyup={checkAvailability}
                id="username"
                placeholder="Username minimum three characters"
                minlength="3"
                maxlength="33"
            />
            <p id="username-helper" class="mt-1 text-sm {helper?.ok ? 'text-green-700' : 'text-red-800'}">
                {helper?.text ?? ''}
            </p>
        </div>
        <div>
            <label for="email" class="label">Email</label>
            <input id="email" class="input" type="email" required bind:value={email} minlength="6" maxlength="256" />
        </div>
        <p class="text-sm text-gray-600">
            We’ll never share your details. Read our
            <a href="/privacy-policy" class="link">Privacy Policy</a>.
        </p>
        <div class="flex justify-center">
            <button class="btn-primary" type="submit" name="action">Submit</button>
        </div>
    </form>
</div>
