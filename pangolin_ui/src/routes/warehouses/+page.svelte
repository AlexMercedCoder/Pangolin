<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import DataTable from '$lib/components/ui/DataTable.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Card from '$lib/components/ui/Card.svelte';
	import { warehousesApi, type Warehouse } from '$lib/api/warehouses';
	import { tenantStore } from '$lib/stores/tenant';
	import { notifications } from '$lib/stores/notifications';

	import Modal from '$lib/components/ui/Modal.svelte';

	let warehouses: Warehouse[] = [];
	let loading = true;
	let showDeleteModal = false;
	let warehouseToDelete: Warehouse | null = null;
	let deleting = false;

	const columns = [
		{ key: 'name', label: 'Name', sortable: true },
		{ key: 'storage_config.type', label: 'Type', sortable: true, width: '120px' },
		{ key: 'storage_config.bucket', label: 'Bucket/Container', sortable: false },
		{ key: 'storage_config.region', label: 'Region', sortable: false, width: '150px' },
		{ key: 'use_sts', label: 'Auth', sortable: false, width: '100px' },
		{ key: 'actions', label: 'Actions', sortable: false },
	];

	// Reload when tenant changes
	$: if ($tenantStore.selectedTenantId || $tenantStore.selectedTenantId === null) {
		loadWarehouses();
	}

	async function loadWarehouses() {
		loading = true;
		try {
			warehouses = await warehousesApi.list();
		} catch (error: any) {
			notifications.error(`Failed to load warehouses: ${error.message}`);
			warehouses = [];
		} finally {
			loading = false;
		}
	}

	function handleRowClick(event: CustomEvent) {
		const warehouse = event.detail;
		goto(`/warehouses/${encodeURIComponent(warehouse.name)}`);
	}

	function confirmDelete(warehouse: Warehouse) {
		warehouseToDelete = warehouse;
		showDeleteModal = true;
	}

	async function handleDelete() {
		if (!warehouseToDelete) return;
		
		deleting = true;
		try {
			await warehousesApi.delete(warehouseToDelete.name);
			notifications.success(`Warehouse "${warehouseToDelete.name}" deleted successfully`);
			warehouses = warehouses.filter(w => w.name !== warehouseToDelete?.name);
			showDeleteModal = false;
			warehouseToDelete = null;
		} catch (error: any) {
			notifications.error(`Failed to delete warehouse: ${error.message}`);
		} finally {
			deleting = false;
		}
	}

	function getStorageType(warehouse: any): string {
		// Infer type from property names since 'type' field doesn't exist
		if (warehouse.storage_config?.['s3.bucket']) return 's3';
		if (warehouse.storage_config?.['adls.account-name']) return 'azure';
		if (warehouse.storage_config?.['gcs.bucket']) return 'gcs';
		return 'unknown';
	}

	function getBucketOrContainer(warehouse: any): string {
		return warehouse.storage_config?.['s3.bucket'] 
			|| warehouse.storage_config?.['azure.container']
			|| warehouse.storage_config?.['gcs.bucket'] 
			|| '-';
	}

	function getRegion(warehouse: any): string {
		return warehouse.storage_config?.['s3.region'] || '-';
	}
</script>

<svelte:head>
	<title>Warehouses - Pangolin</title>
</svelte:head>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-3xl font-bold text-gray-900 dark:text-white">Warehouses</h1>
			<p class="mt-2 text-gray-600 dark:text-gray-400">
				Manage storage warehouses for your Iceberg catalogs
			</p>
		</div>
		<Button on:click={() => goto('/warehouses/new')}>
			<span class="text-lg mr-2">+</span>
			Create Warehouse
		</Button>
	</div>

	<Card>
		<DataTable
			{columns}
			data={warehouses}
			{loading}
			emptyMessage="No warehouses found. Create your first warehouse to get started."
			searchPlaceholder="Search warehouses..."
			on:rowClick={handleRowClick}
		>
			<svelte:fragment slot="cell" let:row let:column>
				{#if column.key === 'storage_config.type'}
					<span
						class="px-2 py-1 text-xs font-medium rounded-full
						{getStorageType(row) === 's3'
							? 'bg-orange-100 dark:bg-orange-900/20 text-orange-800 dark:text-orange-200'
							: getStorageType(row) === 'azure'
							? 'bg-blue-100 dark:bg-blue-900/20 text-blue-800 dark:text-blue-200'
							: 'bg-green-100 dark:bg-green-900/20 text-green-800 dark:text-green-200'}"
					>
						{getStorageType(row).toUpperCase()}
					</span>
				{:else if column.key === 'storage_config.bucket'}
					{getBucketOrContainer(row)}
				{:else if column.key === 'storage_config.region'}
					{getRegion(row)}
				{:else if column.key === 'use_sts'}
					<span
						class="px-2 py-1 text-xs font-medium rounded-full
						{row.use_sts
							? 'bg-success-100 dark:bg-success-900/20 text-success-800 dark:text-success-200'
							: 'bg-gray-100 dark:bg-gray-700 text-gray-800 dark:text-gray-200'}"
					>
						{row.use_sts ? 'IAM' : 'Static'}
					</span>
				{:else if column.key === 'actions'}
					<div class="flex items-center gap-2" on:click|stopPropagation>
						<Button 
							size="sm" 
							variant="secondary" 
							on:click={() => goto(`/warehouses/${encodeURIComponent(row.name)}/edit`)}
							title="Edit Warehouse"
						>
							<span class="material-icons text-sm">edit</span>
						</Button>
						<Button 
							size="sm" 
							variant="error" 
							on:click={() => confirmDelete(row)}
							title="Delete Warehouse"
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
	title="Delete Warehouse"
	on:close={() => showDeleteModal = false}
>
	<div class="space-y-4">
		<p class="text-gray-600 dark:text-gray-300">
			Are you sure you want to delete the warehouse <strong>{warehouseToDelete?.name}</strong>? This action cannot be undone.
		</p>
	</div>
	<div slot="footer" class="flex justify-end gap-3">
		<Button variant="ghost" on:click={() => showDeleteModal = false}>Cancel</Button>
		<Button variant="error" loading={deleting} on:click={handleDelete}>Delete Warehouse</Button>
	</div>
</Modal>
