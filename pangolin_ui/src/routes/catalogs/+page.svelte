<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import DataTable from '$lib/components/ui/DataTable.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Card from '$lib/components/ui/Card.svelte';
	import { catalogsApi, type Catalog } from '$lib/api/catalogs';
	import { tenantStore } from '$lib/stores/tenant';
	import { tenantStore } from '$lib/stores/tenant';
	import { notifications } from '$lib/stores/notifications';
    import { isTenantAdmin } from '$lib/stores/auth';

	let catalogs: Catalog[] = [];
	let loading = true;

	const columns = [
		{ key: 'name', label: 'Name', sortable: true },
		{ key: 'warehouse_name', label: 'Warehouse', sortable: true },
		{ key: 'storage_location', label: 'Storage Location', sortable: false },
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

	function handleRowClick(event: CustomEvent) {
		const catalog = event.detail;
		goto(`/catalogs/${encodeURIComponent(catalog.name)}`);
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
        {#if $isTenantAdmin}
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
					<code class="text-xs bg-gray-100 dark:bg-gray-800 px-2 py-1 rounded">
						{row[column.key]}
					</code>
				{:else}
					{row[column.key] ?? '-'}
				{/if}
			</svelte:fragment>
		</DataTable>
	</Card>
</div>
