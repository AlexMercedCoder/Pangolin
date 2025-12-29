<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Card from '$lib/components/ui/Card.svelte';
	import DataTable from '$lib/components/ui/DataTable.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import { serviceUserStore, filteredServiceUsers } from '$lib/stores/service_users';
	import type { ServiceUser } from '$lib/api/service_users';
	
	import CreateServiceUserModal from '$lib/components/admin/CreateServiceUserModal.svelte';
	import RotateKeyModal from '$lib/components/admin/RotateKeyModal.svelte';
	import ApiKeyDisplayModal from '$lib/components/admin/ApiKeyDisplayModal.svelte';

	let loading = true;

	// Modals State
	let showCreateModal = false;
	let showApiKeyModal = false;
	let showRotateModal = false;
	let showDeleteModal = false;

	// Selected Item State
	let selectedUser: ServiceUser | null = null;
	
	// API Key Display State
	let displayedApiKey = '';
	let displayedKeyName = '';

	const columns = [
		{ key: 'name', label: 'Name', sortable: true },
		{ key: 'role', label: 'Role', sortable: true },
		{ key: 'active', label: 'Status', sortable: true },
		{ key: 'expires_at', label: 'Expires', sortable: true },
		{ key: 'actions', label: 'Actions', sortable: false }
	];

	onMount(async () => {
		loading = true;
		await serviceUserStore.load();
		loading = false;
	});

	// Handlers
	function handleKeyGenerated(key: string, name: string) {
		displayedApiKey = key;
		displayedKeyName = name;
		showApiKeyModal = true;
	}

	function confirmRotate(user: ServiceUser) {
		selectedUser = user;
		showRotateModal = true;
	}

	function confirmDelete(user: ServiceUser) {
		selectedUser = user;
		showDeleteModal = true;
	}

	async function handleDelete() {
		if (selectedUser) {
			await serviceUserStore.deleteUser(selectedUser.id);
			showDeleteModal = false;
			selectedUser = null;
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
		<Button on:click={() => showCreateModal = true}>
			<span class="text-lg mr-2">+</span>
			Create Service User
		</Button>
	</div>

	<Card>
		<DataTable
			{columns}
			data={$filteredServiceUsers}
			{loading}
			emptyMessage="No service users found."
			searchPlaceholder="Search service users..."
			bind:searchValue={$serviceUserStore.searchQuery}
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

<!-- Modals -->
<CreateServiceUserModal 
	bind:open={showCreateModal} 
	onSuccess={handleKeyGenerated} 
/>

<RotateKeyModal 
	bind:open={showRotateModal} 
	user={selectedUser} 
	onSuccess={handleKeyGenerated} 
/>

<ApiKeyDisplayModal 
	bind:open={showApiKeyModal} 
	apiKey={displayedApiKey} 
	keyName={displayedKeyName} 
/>

<Modal bind:open={showDeleteModal} title="Confirm Delete">
	<div class="space-y-4">
		<p class="text-gray-600 dark:text-gray-400">
			Are you sure you want to delete service user <strong>{selectedUser?.name}</strong>? This action cannot be undone.
		</p>
	</div>
	<div slot="footer">
		<Button variant="secondary" on:click={() => showDeleteModal = false}>Cancel</Button>
		<Button 
			variant="error" 
			on:click={handleDelete}
		>
			Delete
		</Button>
	</div>
</Modal>
