<script lang="ts">
	import Button from '$lib/components/ui/Button.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import Select from '$lib/components/ui/Select.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import { serviceUserStore } from '$lib/stores/service_users';

	export let open = false;
	export let onSuccess: (apiKey: string, name: string) => void;

	let name = '';
	let description = '';
	let role = 'tenant-user';
	let expiresIn: number | null = null;
	let processing = false;

	const roleOptions = [
		{ value: 'tenant-user', label: 'Tenant User (Read/Write)' },
		{ value: 'tenant-admin', label: 'Tenant Admin (Full Access)' }
	];

	async function handleSubmit() {
		if (!name) return;
		
		processing = true;
		try {
			const response = await serviceUserStore.create({
				name,
				description: description || undefined,
				role: role as any,
				expires_in_days: expiresIn || undefined
			});
			
			// Reset form
			name = '';
			description = '';
			role = 'tenant-user';
			expiresIn = null;
			open = false;

			// Callback with key
			onSuccess(response.api_key, response.name);
		} finally {
			processing = false;
		}
	}
</script>

<Modal bind:open title="Create Service User">
	<div class="space-y-4">
		<Input
			label="Name"
			bind:value={name}
			placeholder="e.g., ci-pipeline-bot"
			required
		/>
		
		<div>
			<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Description</label>
			<Input
				bind:value={description}
				placeholder="Optional description"
			/>
		</div>

		<Select
			label="Role"
			bind:value={role}
			options={roleOptions}
		/>

		<div>
			<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Expires In (Days)</label>
			<input
				type="number"
				bind:value={expiresIn}
				min="1"
				placeholder="Leave empty for no expiry"
				class="w-full px-3 py-2 border rounded-md dark:bg-gray-800 dark:border-gray-700 dark:text-white focus:ring-primary-500 focus:border-primary-500"
			/>
		</div>
	</div>
	<div slot="footer">
		<Button variant="secondary" on:click={() => open = false}>Cancel</Button>
		<Button 
			variant="primary" 
			disabled={!name || processing}
			on:click={handleSubmit}
		>
			{processing ? 'Creating...' : 'Create'}
		</Button>
	</div>
</Modal>
