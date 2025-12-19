<script lang="ts">
	import { onMount } from 'svelte';
	import { 
		serviceUsersApi, 
		type ServiceUser, 
		type CreateServiceUserRequest,
		type UserRole 
	} from '$lib/api/service_users';
	import Button from '$lib/components/ui/Button.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import Select from '$lib/components/ui/Select.svelte';
	import Card from '$lib/components/ui/Card.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import DataTable from '$lib/components/ui/DataTable.svelte';
	import { notifications } from '$lib/stores/notifications';

	let serviceUsers: ServiceUser[] = [];
	let loading = true;
	let processing = false;

	// Create Modal State
	let showCreateModal = false;
	let newName = '';
	let newDescription = '';
	let newRole: UserRole = 'tenant-user';
	let newExpiresIn: number | null = null;

	// API Key Modal State
	let showApiKeyModal = false;
	let generatedApiKey = '';
	let generatedKeyName = '';

	// Delete Confirmation State
	let showDeleteModal = false;
	let userToDelete: ServiceUser | null = null;

	// Rotate Confirmation State
	let showRotateModal = false;
	let userToRotate: ServiceUser | null = null;

	const roleOptions = [
		{ value: 'tenant-user', label: 'Tenant User (Read/Write)' },
		{ value: 'tenant-admin', label: 'Tenant Admin (Full Access)' }
	];

	const columns = [
		{ key: 'name', label: 'Name', sortable: true },
		{ key: 'role', label: 'Role', sortable: true },
		{ key: 'active', label: 'Status', sortable: true },
		{ key: 'expires_at', label: 'Expires', sortable: true },
		{ key: 'actions', label: 'Actions', sortable: false }
	];

	onMount(loadServiceUsers);

	async function loadServiceUsers() {
		loading = true;
		try {
			serviceUsers = await serviceUsersApi.list();
		} catch (error: any) {
			notifications.error(`Failed to load service users: ${error.message}`);
		} finally {
			loading = false;
		}
	}

	function openCreateModal() {
		newName = '';
		newDescription = '';
		newRole = 'tenant-user';
		newExpiresIn = null;
		showCreateModal = true;
	}

	async function handleCreate() {
		if (!newName) return;
		
		processing = true;
		try {
			const response = await serviceUsersApi.create({
				name: newName,
				description: newDescription || undefined,
				role: newRole,
				expires_in_days: newExpiresIn || undefined
			});

			generatedApiKey = response.api_key;
			generatedKeyName = response.name;
			showCreateModal = false;
			showApiKeyModal = true;
			notifications.success('Service user created successfully');
			await loadServiceUsers();
		} catch (error: any) {
			notifications.error(`Failed to create service user: ${error.message}`);
		} finally {
			processing = false;
		}
	}

	function confirmDelete(user: ServiceUser) {
		userToDelete = user;
		showDeleteModal = true;
	}

	async function handleDelete() {
		if (!userToDelete) return;
		
		processing = true;
		try {
			await serviceUsersApi.delete(userToDelete.id);
			notifications.success(`Service user "${userToDelete.name}" deleted`);
			showDeleteModal = false;
			userToDelete = null;
			await loadServiceUsers();
		} catch (error: any) {
			notifications.error(`Failed to delete service user: ${error.message}`);
		} finally {
			processing = false;
		}
	}

	function confirmRotate(user: ServiceUser) {
		userToRotate = user;
		showRotateModal = true;
	}

	async function handleRotate() {
		if (!userToRotate) return;
		
		processing = true;
		try {
			const response = await serviceUsersApi.rotateApiKey(userToRotate.id);
			generatedApiKey = response.api_key;
			generatedKeyName = userToRotate.name;
			
			showRotateModal = false;
			userToRotate = null;
			
			showApiKeyModal = true;
			notifications.success('API key rotated successfully');
		} catch (error: any) {
			notifications.error(`Failed to rotate API key: ${error.message}`);
		} finally {
			processing = false;
		}
	}

	async function copyApiKey() {
		try {
			await navigator.clipboard.writeText(generatedApiKey);
			notifications.success('API key copied to clipboard');
		} catch (err) {
			notifications.error('Failed to copy API key');
		}
	}

	function formatDate(dateStr?: string) {
		if (!dateStr) return 'Never';
		return new Date(dateStr).toLocaleDateString();
	}
</script>

<svelte:head>
	<title>Service Users - Pangolin</title>
</svelte:head>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-3xl font-bold text-gray-900 dark:text-white">Service Users</h1>
			<p class="mt-2 text-gray-600 dark:text-gray-400">
				Manage service accounts for machine-to-machine authentication.
			</p>
		</div>
		<Button on:click={openCreateModal}>
			<span class="text-lg mr-2">+</span>
			Create Service User
		</Button>
	</div>

	<Card>
		<DataTable
			{columns}
			data={serviceUsers}
			{loading}
			emptyMessage="No service users found."
			searchPlaceholder="Search service users..."
		>
			<svelte:fragment slot="cell" let:row let:column>
				{#if column.key === 'role'}
					<span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200">
						{row.role}
					</span>
				{:else if column.key === 'active'}
					<span class={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${row.active ? 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200' : 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200'}`}>
						{row.active ? 'Active' : 'Inactive'}
					</span>
				{:else if column.key === 'expires_at'}
					<span class="text-gray-600 dark:text-gray-400 text-sm">
						{formatDate(row.expires_at)}
					</span>
				{:else if column.key === 'actions'}
					<div class="flex items-center gap-2">
						<Button 
							size="sm" 
							variant="secondary" 
							title="Rotate API Key"
							on:click={() => confirmRotate(row)}
						>
							<span class="material-icons text-sm">refresh</span>
						</Button>
						<Button 
							size="sm" 
							variant="error" 
							title="Delete"
							on:click={() => confirmDelete(row)}
						>
							<span class="material-icons text-sm">delete</span>
						</Button>
					</div>
				{:else}
					{row[column.key]}
				{/if}
			</svelte:fragment>
		</DataTable>
	</Card>
</div>

<!-- Create Modal -->
<Modal bind:open={showCreateModal} title="Create Service User">
	<div class="space-y-4">
		<Input
			label="Name"
			bind:value={newName}
			placeholder="e.g., ci-pipeline-bot"
			required
		/>
		
		<div>
			<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Description</label>
			<Input
				bind:value={newDescription}
				placeholder="Optional description"
			/>
		</div>

		<Select
			label="Role"
			bind:value={newRole}
			options={roleOptions}
		/>

		<div>
			<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Expires In (Days)</label>
			<input
				type="number"
				bind:value={newExpiresIn}
				min="1"
				placeholder="Leave empty for no expiry"
				class="w-full px-3 py-2 border rounded-md dark:bg-gray-800 dark:border-gray-700 dark:text-white focus:ring-primary-500 focus:border-primary-500"
			/>
		</div>
	</div>
	<div slot="footer">
		<Button variant="secondary" on:click={() => showCreateModal = false}>Cancel</Button>
		<Button 
			variant="primary" 
			disabled={!newName || processing}
			on:click={handleCreate}
		>
			{processing ? 'Creating...' : 'Create'}
		</Button>
	</div>
</Modal>

<!-- API Key Display Modal -->
<Modal bind:open={showApiKeyModal} title="API Key Generated" on:close={() => showApiKeyModal = false}>
	<div class="space-y-4">
		<div class="bg-yellow-50 dark:bg-yellow-900/30 p-4 rounded-md border border-yellow-200 dark:border-yellow-800">
			<p class="text-sm text-yellow-800 dark:text-yellow-200 font-medium">
				Warning: This API key will only be shown once. Please copy it and store it securely.
			</p>
		</div>

		<div>
			<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
				API Key for <strong>{generatedKeyName}</strong>
			</label>
			<div class="relative">
				<textarea 
					class="w-full px-3 py-2 border rounded-md dark:bg-gray-800 dark:border-gray-700 dark:text-white font-mono text-sm h-24 pr-12 resize-none"
					readonly
					value={generatedApiKey}
				></textarea>
				<button 
					class="absolute top-2 right-2 p-2 text-gray-500 hover:text-gray-700 dark:hover:text-gray-300 bg-white dark:bg-gray-800 rounded border border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
					on:click={copyApiKey}
					title="Copy to clipboard"
				>
					<span class="material-icons text-sm">content_copy</span>
				</button>
			</div>
		</div>
	</div>
	<div slot="footer">
		<Button on:click={() => showApiKeyModal = false}>Close</Button>
	</div>
</Modal>

<!-- Delete Confirmation Modal -->
<Modal bind:open={showDeleteModal} title="Confirm Delete">
	<div class="space-y-4">
		<p class="text-gray-600 dark:text-gray-400">
			Are you sure you want to delete service user <strong>{userToDelete?.name}</strong>? This action cannot be undone.
		</p>
	</div>
	<div slot="footer">
		<Button variant="secondary" on:click={() => showDeleteModal = false}>Cancel</Button>
		<Button 
			variant="error" 
			disabled={processing}
			on:click={handleDelete}
		>
			{processing ? 'Deleting...' : 'Delete'}
		</Button>
	</div>
</Modal>

<!-- Rotate Confirmation Modal -->
<Modal bind:open={showRotateModal} title="Confirm Key Rotation">
	<div class="space-y-4">
		<p class="text-gray-600 dark:text-gray-400">
			Are you sure you want to rotate the API key for <strong>{userToRotate?.name}</strong>?
		</p>
		<p class="text-red-600 dark:text-red-400 text-sm font-medium">
			The old API key will stop working immediately.
		</p>
	</div>
	<div slot="footer">
		<Button variant="secondary" on:click={() => showRotateModal = false}>Cancel</Button>
		<Button 
			variant="primary" 
			disabled={processing}
			on:click={handleRotate}
		>
			{processing ? 'Rotating...' : 'Rotate Key'}
		</Button>
	</div>
</Modal>
