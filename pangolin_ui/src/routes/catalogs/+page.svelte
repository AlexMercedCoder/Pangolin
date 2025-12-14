<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import DataTable from '$lib/components/ui/DataTable.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Card from '$lib/components/ui/Card.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import Select from '$lib/components/ui/Select.svelte';
	import { catalogsApi, type Catalog, type CreateCatalogRequest } from '$lib/api/catalogs';
	import { warehousesApi } from '$lib/api/warehouses';
	import { notifications } from '$lib/stores/notifications';

	let catalogs: Catalog[] = [];
	let warehouses: any[] = [];
	let loading = true;
	let showCreateModal = false;
	let creating = false;

	// Form state
	let formData: CreateCatalogRequest = {
		name: '',
		warehouse_name: '',
		storage_location: '',
		properties: {},
	};
	let formErrors: Record<string, string> = {};

	const columns = [
		{ key: 'name', label: 'Name', sortable: true },
		{ key: 'warehouse_name', label: 'Warehouse', sortable: true },
		{ key: 'storage_location', label: 'Storage Location', sortable: false },
	];

	onMount(async () => {
		await Promise.all([loadCatalogs(), loadWarehouses()]);
	});

	async function loadCatalogs() {
		loading = true;
		const response = await catalogsApi.list();
		
		if (response.error) {
			notifications.error(`Failed to load catalogs: ${response.error.message}`);
			catalogs = [];
		} else {
			catalogs = response.data || [];
		}
		
		loading = false;
	}

	async function loadWarehouses() {
		const response = await warehousesApi.list();
		if (response.data) {
			warehouses = response.data.map(w => ({ value: w.name, label: w.name }));
		}
	}

	function handleRowClick(event: CustomEvent) {
		const catalog = event.detail;
		goto(`/catalogs/${encodeURIComponent(catalog.name)}`);
	}

	function openCreateModal() {
		formData = {
			name: '',
			warehouse_name: '',
			storage_location: '',
			properties: {},
		};
		formErrors = {};
		showCreateModal = true;
	}

	function validateForm(): boolean {
		formErrors = {};

		if (!formData.name.trim()) {
			formErrors.name = 'Name is required';
		} else if (!/^[a-zA-Z0-9_-]+$/.test(formData.name)) {
			formErrors.name = 'Name can only contain letters, numbers, hyphens, and underscores';
		}

		if (!formData.warehouse_name) {
			formErrors.warehouse_name = 'Warehouse is required';
		}

		if (!formData.storage_location.trim()) {
			formErrors.storage_location = 'Storage location is required';
		}

		return Object.keys(formErrors).length === 0;
	}

	async function handleCreate() {
		if (!validateForm()) return;

		creating = true;
		const response = await catalogsApi.create(formData);

		if (response.error) {
			notifications.error(`Failed to create catalog: ${response.error.message}`);
		} else {
			notifications.success(`Catalog "${formData.name}" created successfully`);
			showCreateModal = false;
			await loadCatalogs();
		}

		creating = false;
	}

	// Auto-fill storage location when warehouse is selected
	$: if (formData.warehouse_name && !formData.storage_location) {
		const warehouse = warehouses.find(w => w.value === formData.warehouse_name);
		if (warehouse) {
			// Auto-generate storage location based on warehouse
			formData.storage_location = `s3://bucket/${formData.name || 'catalog'}`;
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
		<Button on:click={openCreateModal}>
			<span class="text-lg mr-2">+</span>
			Create Catalog
		</Button>
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

<!-- Create Catalog Modal -->
<Modal bind:open={showCreateModal}>
	<div class="space-y-4">
		<h2 class="text-2xl font-bold text-gray-900 dark:text-white">Create Catalog</h2>
		
		<form on:submit|preventDefault={handleCreate} class="space-y-4">
			<Input
				label="Catalog Name"
				bind:value={formData.name}
				placeholder="my-catalog"
				required
				error={formErrors.name}
			/>

			<Select
				label="Warehouse"
				bind:value={formData.warehouse_name}
				options={warehouses}
				placeholder="Select a warehouse"
				required
				error={formErrors.warehouse_name}
			/>

			<Input
				label="Storage Location"
				bind:value={formData.storage_location}
				placeholder="s3://my-bucket/my-catalog/ or s3://different-bucket/path/"
				required
				error={formErrors.storage_location}
			/>
			<div class="p-4 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
				<p class="text-sm text-blue-800 dark:text-blue-200">
					<strong>💡 Flexible Storage:</strong> You can specify any S3 path here - it doesn't have to be in the warehouse's bucket. The warehouse provides authentication, while this location determines where your catalog's data is stored.
				</p>
			</div>

			<div class="pt-4 border-t border-gray-200 dark:border-gray-700 flex items-center gap-3 justify-end">
				<Button
					type="button"
					variant="secondary"
					on:click={() => (showCreateModal = false)}
					disabled={creating}
				>
					Cancel
				</Button>
				<Button type="submit" disabled={creating}>
					{creating ? 'Creating...' : 'Create Catalog'}
				</Button>
			</div>
		</form>
	</div>
</Modal>
