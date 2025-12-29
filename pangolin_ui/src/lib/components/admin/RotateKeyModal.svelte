<script lang="ts">
	import Button from '$lib/components/ui/Button.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import { serviceUserStore } from '$lib/stores/service_users';
	import type { ServiceUser } from '$lib/api/service_users';

	export let open = false;
	export let user: ServiceUser | null = null;
	export let onSuccess: (apiKey: string, name: string) => void;

	let processing = false;

	async function handleRotate() {
		if (!user) return;
		
		processing = true;
		try {
			const response = await serviceUserStore.rotateKey(user.id);
			open = false;
			onSuccess(response.api_key, user.name);
		} finally {
			processing = false;
		}
	}
</script>

<Modal bind:open title="Confirm Key Rotation">
	<div class="space-y-4">
		<p class="text-gray-600 dark:text-gray-400">
			Are you sure you want to rotate the API key for <strong>{user?.name}</strong>?
		</p>
		<p class="text-red-600 dark:text-red-400 text-sm font-medium">
			The old API key will stop working immediately.
		</p>
	</div>
	<div slot="footer">
		<Button variant="secondary" on:click={() => open = false}>Cancel</Button>
		<Button 
			variant="primary" 
			disabled={processing}
			on:click={handleRotate}
		>
			{processing ? 'Rotating...' : 'Rotate Key'}
		</Button>
	</div>
</Modal>
