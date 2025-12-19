<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import DataTable from '$lib/components/ui/DataTable.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Card from '$lib/components/ui/Card.svelte';
	import { tenantsApi, type Tenant } from '$lib/api/tenants';
	import { notifications } from '$lib/stores/notifications';
	import Modal from '$lib/components/ui/Modal.svelte';

	let tenants: Tenant[] = [];
	let loading = true;
	let showDeleteModal = false;
	let tenantToDelete: Tenant | null = null;
	let deleting = false;

	const columns = [
		{ key: 'name', label: 'Name', sortable: true },
		{ key: 'id', label: 'ID', sortable: true, width: '300px' },
		{ key: 'actions', label: 'Actions', sortable: false },
	];

	onMount(async () => {
		await loadTenants();
	});

	async function loadTenants() {
		loading = true;
		try {
			tenants = await tenantsApi.list();
		} catch (error: any) {
			notifications.error(`Failed to load tenants: ${error.message}`);
			tenants = [];
		} finally {
			loading = false;
		}
	}

	function handleRowClick(event: CustomEvent) {
		const tenant = event.detail;
		goto(`/tenants/${tenant.id}`);
	}

	function confirmDelete(tenant: Tenant) {
		tenantToDelete = tenant;
		showDeleteModal = true;
	}

	async function handleDelete() {
		if (!tenantToDelete) return;
		
		deleting = true;
		try {
			await tenantsApi.delete(tenantToDelete.id);
			notifications.success(`Tenant "${tenantToDelete.name}" deleted successfully`);
			tenants = tenants.filter(t => t.id !== tenantToDelete?.id);
			showDeleteModal = false;
			tenantToDelete = null;
		} catch (error: any) {
			notifications.error(`Failed to delete tenant: ${error.message}`);
		} finally {
			deleting = false;
		}
	}
</script>

<svelte:head>
	<title>Tenants - Pangolin</title>
</svelte:head>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-3xl font-bold text-gray-900 dark:text-white">Tenants</h1>
			<p class="mt-2 text-gray-600 dark:text-gray-400">
				Manage multi-tenant isolation and organization
			</p>
		</div>
		<Button on:click={() => goto('/tenants/new')}>
			<span class="text-lg mr-2">+</span>
			Create Tenant
		</Button>
	</div>

	<Card>
		<DataTable
			{columns}
			data={tenants}
			{loading}
			emptyMessage="No tenants found. Create your first tenant to get started."
			searchPlaceholder="Search tenants..."
			on:rowClick={handleRowClick}
		>
			<svelte:fragment slot="cell" let:row let:column>
				{#if column.key === 'id'}
					<span class="font-mono text-sm">{row.id}</span>
				{:else if column.key === 'actions'}
					<div class="flex items-center gap-2" on:click|stopPropagation>
						<Button 
							size="sm" 
							variant="secondary" 
							on:click={() => goto(`/tenants/${row.id}/edit`)}
							title="Edit Tenant"
						>
							<span class="material-icons text-sm">edit</span>
						</Button>
						<Button 
							size="sm" 
							variant="error" 
							on:click={() => confirmDelete(row)}
							title="Delete Tenant"
						>
							<span class="material-icons text-sm">delete</span>
						</Button>
					</div>
				{:else}
					{row[column.key] ?? '-'}
				{/if}
			</svelte:fragment>
		</DataTable>
	</Card>
</div>

<Modal
	open={showDeleteModal}
	title="Delete Tenant"
	on:close={() => showDeleteModal = false}
>
	<div class="space-y-4">
		<p class="text-gray-600 dark:text-gray-300">
			Are you sure you want to delete the tenant <strong>{tenantToDelete?.name}</strong>? This action cannot be undone and will delete all associated data.
		</p>
	</div>
	<div slot="footer" class="flex justify-end gap-3">
		<Button variant="ghost" on:click={() => showDeleteModal = false}>Cancel</Button>
		<Button variant="error" loading={deleting} on:click={handleDelete}>Delete Tenant</Button>
	</div>
</Modal>
