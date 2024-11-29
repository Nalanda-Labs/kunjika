<script>
	import { page } from '$app/stores';
	import { browser } from '$app/environment';

	export let id;

	function deleteProfile() {
		if (parseInt(id) !== parseInt($page.data.user.id)) {
            alert("Only self can delete self's profile!");
            return;
		}

		if (browser) {
			if (confirm('Are you sure that you want to delete your profile?')) {
				document.forms['delete-profile'].submit();
			}
		}
	}
</script>

<div class="row justify-content-center align-items-center" style="margin-top:20px">
	<div class="row col-12">
		{#if id == $page.data.user.id}
			<p style="color: #f00;font-weight:500">
				Deleting yourself will delete your email and user from the application. All your user names
				will be changed to "u{id}". There is no way to undo this operation. However, you will be
				able to reuse the email associated with this account to create a new login but all your
				contributions in this account will not be associated with a new account with same email.
			</p>
			<div class="b-wrapper">
				<form
					style="margin-top:5px;"
					method="POST"
					id="delete-profile"
					on:submit|preventDefault={() => deleteProfile()}
				>
					<button class="btn btn-danger">Delete Profile</button>
				</form>
			</div>
		{/if}
	</div>
</div>
