<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import DataTable from '$lib/components/ui/DataTable.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Card from '$lib/components/ui/Card.svelte';
	import { catalogsApi, type Catalog } from '$lib/api/catalogs';
	import { tenantStore } from '$lib/stores/tenant';
	import { authStore } from '$lib/stores/auth';

	import { notifications } from '$lib/stores/notifications';
	import Modal from '$lib/components/ui/Modal.svelte';

	let catalogs: Catalog[] = [];
	let loading = true;
	let showDeleteModal = false;
	let catalogToDelete: Catalog | null = null;
	let deleting = false;

	const columns = [
		{ key: 'name', label: 'Name', sortable: true },
		{ key: 'catalog_type', label: 'Type', sortable: true },
		{ key: 'warehouse_name', label: 'Warehouse', sortable: true },
		{ key: 'storage_location', label: 'Location / URL', sortable: false },
		{ key: 'actions', label: 'Actions', sortable: false },
	];

	// Reload when tenant changes
	$: if ($tenantStore.selectedTenantId || $tenantStore.selectedTenantId === null) {
		loadCatalogs();
	}

	async function loadCatalogs() {
		loading = true;
		try {
			catalogs = await catalogsApi.list();
		} catch (error: any) {
			notifications.error(`Failed to load catalogs: ${error.message}`);
			catalogs = [];
		} finally {
			loading = false;
		}
	}

	async function testConnection(catalog: Catalog) {
		loading = true;
		try {
			await catalogsApi.testConnection(catalog.name);
			notifications.success(`Successfully connected to federated catalog "${catalog.name}"`);
		} catch (error: any) {
			notifications.error(`Connection failed for "${catalog.name}": ${error.message}`);
		} finally {
			loading = false;
		}
	}

	function handleRowClick(event: CustomEvent) {
		const catalog = event.detail;
		goto(`/catalogs/${encodeURIComponent(catalog.name)}`);
	}

	function confirmDelete(catalog: Catalog) {
		catalogToDelete = catalog;
		showDeleteModal = true;
	}

	async function handleDelete() {
		if (!catalogToDelete) return;
		
		deleting = true;
		try {
			await catalogsApi.delete(catalogToDelete.name);
			notifications.success(`Catalog "${catalogToDelete.name}" deleted successfully`);
			catalogs = catalogs.filter(c => c.name !== catalogToDelete?.name);
			showDeleteModal = false;
			catalogToDelete = null;
		} catch (error: any) {
			notifications.error(`Failed to delete catalog: ${error.message}`);
		} finally {
			deleting = false;
		}
	}
</script>

<svelte:head>
	<title>Catalogs - Pangolin</title>
</svelte:head>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-3xl font-bold text-gray-900 dark:text-white">Catalogs</h1>
			<p class="mt-2 text-gray-600 dark:text-gray-400">
				Manage Iceberg catalogs and their configurations
			</p>
		</div>
        {#if $authStore.user?.role === 'Root' || $authStore.user?.role === 'TenantAdmin'}
		<Button on:click={() => goto('/catalogs/new')}>
			<span class="text-lg mr-2">+</span>
			Create Catalog
		</Button>
        {/if}
	</div>

	<Card>
		<DataTable
			{columns}
			data={catalogs}
			{loading}
			emptyMessage="No catalogs found. Create your first catalog to get started."
			searchPlaceholder="Search catalogs..."
			on:rowClick={handleRowClick}
		>
			<svelte:fragment slot="cell" let:row let:column>
				{#if column.key === 'storage_location'}
					<code class="text-xs bg-gray-100 dark:bg-gray-800 px-2 py-1 rounded max-w-xs truncate block" title={row.catalog_type === 'Federated' ? row.federated_config?.base_url : row.storage_location}>
						{row.catalog_type === 'Federated' ? row.federated_config?.base_url : row.storage_location}
					</code>
				{:else if column.key === 'catalog_type'}
					<span class={`inline-flex items-center px-2 py-0.5 rounded text-xs font-medium ${row.catalog_type === 'Federated' ? 'bg-purple-100 text-purple-800 dark:bg-purple-900/30 dark:text-purple-300' : 'bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-300'}`}>
						{row.catalog_type}
					</span>
				{:else if column.key === 'actions'}
					<div class="flex items-center gap-2" on:click|stopPropagation>
						{#if row.catalog_type === 'Federated'}
							<Button 
								size="sm" 
								variant="secondary" 
								on:click={() => testConnection(row)}
								title="Test Connection"
							>
								<span class="material-icons text-sm">network_check</span>
							</Button>
						{/if}
						<Button 
							size="sm" 
							variant="secondary" 
							on:click={() => goto(`/catalogs/${encodeURIComponent(row.name)}/edit`)}
							title="Edit Catalog"
						>
							<span class="material-icons text-sm">edit</span>
						</Button>
						<Button 
							size="sm" 
							variant="error" 
							on:click={() => confirmDelete(row)}
							title="Delete Catalog"
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
	title="Delete Catalog"
	on:close={() => showDeleteModal = false}
>
	<div class="space-y-4">
		<p class="text-gray-600 dark:text-gray-300">
			Are you sure you want to delete the catalog <strong>{catalogToDelete?.name}</strong>? This action cannot be undone.
		</p>
	</div>
	<div slot="footer" class="flex justify-end gap-3">
		<Button variant="ghost" on:click={() => showDeleteModal = false}>Cancel</Button>
		<Button variant="error" loading={deleting} on:click={handleDelete}>Delete Catalog</Button>
	</div>
</Modal>
