<script lang="ts">
    let { id }: { id: number | string } = $props();
    let form = $state<HTMLFormElement>();

    function confirmDelete(e: SubmitEvent) {
        e.preventDefault();
        if (confirm('Are you sure that you want to delete your profile?')) form?.submit();
    }
</script>

<div class="mt-4 space-y-4">
    <p class="font-medium text-red-600">
        Deleting yourself will delete your email and user from the application. Your user name will be
        changed to "u{id}". There is no way to undo this operation. You will be able to reuse the email
        of this account to create a new login, but your contributions in this account will not be
        associated with the new account.
    </p>
    <form bind:this={form} method="POST" id="delete-profile" onsubmit={confirmDelete}>
        <button class="btn-danger">Delete Profile</button>
    </form>
</div>
