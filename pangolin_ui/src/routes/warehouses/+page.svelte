<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import DataTable from '$lib/components/ui/DataTable.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Card from '$lib/components/ui/Card.svelte';
	import { warehousesApi, type Warehouse } from '$lib/api/warehouses';
	import { notifications } from '$lib/stores/notifications';

	let warehouses: Warehouse[] = [];
	let loading = true;

	const columns = [
		{ key: 'name', label: 'Name', sortable: true },
		{ key: 'storage_config.type', label: 'Type', sortable: true, width: '120px' },
		{ key: 'storage_config.bucket', label: 'Bucket/Container', sortable: false },
		{ key: 'storage_config.region', label: 'Region', sortable: false, width: '150px' },
		{ key: 'use_sts', label: 'Auth', sortable: false, width: '100px' },
	];

	onMount(async () => {
		await loadWarehouses();
	});

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

	function getStorageType(warehouse: Warehouse): string {
		const type = warehouse.storage_config?.type || 'unknown';
		return type.toUpperCase();
	}

	function getBucketOrContainer(warehouse: Warehouse): string {
		return warehouse.storage_config?.bucket || warehouse.storage_config?.container || '-';
	}

	function getRegion(warehouse: Warehouse): string {
		return warehouse.storage_config?.region || '-';
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
						{row.storage_config?.type === 's3'
							? 'bg-orange-100 dark:bg-orange-900/20 text-orange-800 dark:text-orange-200'
							: row.storage_config?.type === 'azure'
							? 'bg-blue-100 dark:bg-blue-900/20 text-blue-800 dark:text-blue-200'
							: 'bg-green-100 dark:bg-green-900/20 text-green-800 dark:text-green-200'}"
					>
						{getStorageType(row)}
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
				{:else}
					{row[column.key] ?? '-'}
				{/if}
			</svelte:fragment>
		</DataTable>
	</Card>
</div>
