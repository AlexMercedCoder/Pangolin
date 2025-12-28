<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import Card from '$lib/components/ui/Card.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import Select from '$lib/components/ui/Select.svelte';
	import { catalogsApi, type Catalog, type UpdateCatalogRequest } from '$lib/api/catalogs';
	import { warehousesApi, type Warehouse } from '$lib/api/warehouses';
	import { notifications } from '$lib/stores/notifications';

	let catalog: Catalog | null = null;
	let warehouses: Warehouse[] = [];
	let loading = true;
	let submitting = false;

	// Form data
	let warehouseName = '';
	let storageLocation = '';
	let errors: Record<string, string> = {};

	$: catalogName = $page.params.name || '';
	
	$: warehouseOptions = [
		{ value: '', label: 'None (No warehouse)' },
		...warehouses.map(w => ({ value: w.name, label: w.name }))
	];

	onMount(async () => {
		await Promise.all([loadCatalog(), loadWarehouses()]);
	});

	async function loadCatalog() {
		if (!catalogName) return;

		loading = true;
		try {
			catalog = await catalogsApi.get(catalogName);
			// Pre-populate form
			warehouseName = catalog.warehouse_name || '';
			storageLocation = catalog.storage_location || '';
            // Mark as manually set initially to prevent auto-overwriting existing custom paths on load
            manualLocation = true;
		} catch (error: any) {
			notifications.error(`Failed to load catalog: ${error.message}`);
			goto('/catalogs');
		}
		loading = false;
	}
	
	async function loadWarehouses() {
		try {
			warehouses = await warehousesApi.list();
		} catch (error: any) {
			console.error('Failed to load warehouses:', error);
			warehouses = [];
		}
	}

	function validateForm(): boolean {
		errors = {};

		if (!storageLocation) {
			errors.storageLocation = 'Storage location is required';
		}

		return Object.keys(errors).length === 0;
	}

    let manualLocation = false;

    // Helper to determine type from config keys
    function getStorageType(config: Record<string, string> | undefined): 's3' | 'azure' | 'gcs' | 's3' {
        if (!config) return 's3';
        if (config['s3.bucket']) return 's3';
        if (config['adls.account-name'] || config['azure.container']) return 'azure';
        if (config['gcs.bucket']) return 'gcs';
        return 's3';
    }

    // Reactive update for storage location
    $: if (warehouseName && !manualLocation) {
        const selected = warehouses.find(w => w.name === warehouseName);
        if (selected?.storage_config) {
            const config = selected.storage_config;
            const bucket = config['s3.bucket'] 
                       || config['adls.container']
                       || config['azure.container'] 
                       || config['gcs.bucket'];
            
            const type = getStorageType(config);
            
            // Only auto-update if we have a valid bucket to form a path
            if (bucket) {
                 if (type === 'azure') {
                    const account = config['adls.account-name'];
                    if (account) {
                        storageLocation = `abfss://${bucket}@${account}.dfs.core.windows.net/${catalog?.name || 'catalog'}`;
                    }
                } else if (type === 's3') {
                    storageLocation = `s3://${bucket}/${catalog?.name || 'catalog'}`;
                } else if (type === 'gcs') {
                    storageLocation = `gs://${bucket}/${catalog?.name || 'catalog'}`;
                }
            }
        }
    }

	async function handleSubmit() {
		if (!validateForm() || !catalogName) return;

		submitting = true;
		try {
			const updateData: UpdateCatalogRequest = {
				storage_location: storageLocation,
			};

			// Only include warehouse_name if it's not empty
			if (warehouseName) {
				updateData.warehouse_name = warehouseName;
			}

			await catalogsApi.update(catalogName, updateData);
			notifications.success(`Catalog "${catalogName}" updated successfully`);
			goto(`/catalogs/${encodeURIComponent(catalogName)}`);
		} catch (error: any) {
			notifications.error(`Failed to update catalog: ${error.message}`);
		}
		submitting = false;
	}
</script>

<svelte:head>
	<title>Edit {catalog?.name || 'Catalog'} - Pangolin</title>
</svelte:head>

<div class="space-y-6">
	<!-- Header -->
	<div>
		<div class="flex items-center gap-3">
			<button
				on:click={() => goto(`/catalogs/${encodeURIComponent(catalogName)}`)}
				class="text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white"
			>
				← Back
			</button>
			<h1 class="text-3xl font-bold text-gray-900 dark:text-white">
				Edit Catalog: {catalog?.name || 'Loading...'}
			</h1>
		</div>
		<p class="mt-2 text-gray-600 dark:text-gray-400">Update catalog configuration</p>
	</div>

	{#if loading}
		<Card>
			<div class="flex items-center justify-center py-12">
				<div class="w-8 h-8 border-3 border-primary-600 border-t-transparent rounded-full animate-spin" />
			</div>
		</Card>
	{:else if catalog}
		<Card>
			<form on:submit|preventDefault={handleSubmit} class="space-y-6">
				<div class="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-4">
					<p class="text-sm text-blue-800 dark:text-blue-200">
						<strong>Note:</strong> Catalog name cannot be changed. To rename a catalog, create a new one and migrate your data.
					</p>
				</div>

				<Input
					label="Catalog Name"
					value={catalog.name}
					disabled
					helpText="Catalog name is immutable"
				/>

				<Select
				label="Warehouse"
				bind:value={warehouseName}
				options={warehouseOptions}
                on:change={() => manualLocation = false}
				placeholder="Select a warehouse..."
				helpText="Optional: Link this catalog to a warehouse for credential vending"
			/>

				<Input
					label="Storage Location"
					bind:value={storageLocation}
                    on:input={() => manualLocation = true}
					error={errors.storageLocation}
					required
					placeholder="s3://bucket/path or /local/path"
					helpText="Base path where Iceberg table data will be stored"
				/>

				<div class="flex items-center gap-3 pt-4 border-t border-gray-200 dark:border-gray-700">
					<Button
						type="button"
						variant="ghost"
						on:click={() => goto(`/catalogs/${encodeURIComponent(catalogName)}`)}
						disabled={submitting}
					>
						Cancel
					</Button>
					<Button type="submit" loading={submitting}>
						{submitting ? 'Updating...' : 'Update Catalog'}
					</Button>
				</div>
			</form>
		</Card>
	{/if}
</div>
