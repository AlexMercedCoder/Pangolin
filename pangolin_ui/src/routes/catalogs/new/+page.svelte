<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { catalogsApi, type CreateCatalogRequest } from '$lib/api/catalogs';
	import { warehousesApi, type Warehouse } from '$lib/api/warehouses';
	import Button from '$lib/components/ui/Button.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import Select from '$lib/components/ui/Select.svelte';
	import Card from '$lib/components/ui/Card.svelte';
	import { notifications } from '$lib/stores/notifications';

	let loading = false;
	let error = '';
	
	let warehouses: { value: string; label: string; full: Warehouse }[] = [];
	let loadingWarehouses = true;

	// Form
	let name = '';
	let warehouseName = '';
	let storageLocation = '';

	onMount(async () => {
		try {
			const wList = await warehousesApi.list();
			warehouses = wList.map(w => ({ 
				value: w.name, 
				label: w.name,
				full: w
			}));
		} catch (e: any) {
			notifications.error('Failed to load warehouses');
		} finally {
			loadingWarehouses = false;
		}
	});

	// Auto-fill storage location logic
	$: if (warehouseName && !storageLocation) {
		const selected = warehouses.find(w => w.value === warehouseName);
		if (selected) {
			// Simple heuristic for default path
			const w = selected.full;
			const bucket = w.storage_config?.bucket || w.storage_config?.container || 'bucket';
			const type = w.storage_config?.type || 's3';
			
			if (type === 'azure') {
				storageLocation = `abfss://${bucket}@${w.storage_config?.account_name}.dfs.core.windows.net/${name || 'catalog'}`;
			} else if (type === 'gcs') {
				storageLocation = `gs://${bucket}/${name || 'catalog'}`;
			} else {
				storageLocation = `s3://${bucket}/${name || 'catalog'}`;
			}
		}
	}

	async function handleSubmit() {
		loading = true;
		error = '';

		try {
			const request: CreateCatalogRequest = {
				name,
				warehouse_name: warehouseName || undefined,
				storage_location: storageLocation,
				properties: {}
			};

			await catalogsApi.create(request);
			notifications.success(`Catalog "${name}" created successfully`);
			goto('/catalogs');
		} catch (e: any) {
			error = e.message || 'Failed to create catalog';
			notifications.error(error);
		} finally {
			loading = false;
		}
	}
</script>

<svelte:head>
	<title>Create Catalog - Pangolin</title>
</svelte:head>

<div class="max-w-2xl mx-auto space-y-6">
	<div>
		<h1 class="text-3xl font-bold text-gray-900 dark:text-white">Create Catalog</h1>
		<p class="mt-2 text-gray-600 dark:text-gray-400">
			Create a new Iceberg catalog backed by a warehouse
		</p>
	</div>

	<Card>
		<form on:submit|preventDefault={handleSubmit} class="space-y-6">
			{#if error}
				<div class="p-4 bg-error-50 dark:bg-error-900/20 text-error-700 dark:text-error-200 rounded-lg">
					{error}
				</div>
			{/if}

			<div class="grid gap-6">
				<Input
					label="Catalog Name"
					bind:value={name}
					placeholder="my-catalog"
					required
					disabled={loading}
				/>

				<div>
					<Select
						label="Warehouse"
						bind:value={warehouseName}
						options={warehouses}
						placeholder={loadingWarehouses ? "Loading warehouses..." : "Select a warehouse (Optional)"}
						disabled={loading || loadingWarehouses}
					/>
					{#if !warehouseName}
						<div class="mt-2 p-3 bg-yellow-50 dark:bg-yellow-900/20 text-yellow-700 dark:text-yellow-200 text-sm rounded-lg flex items-start gap-2">
							<span class="text-lg">⚠️</span>
							<p>Without a warehouse, you must manually specify storage credentials and details in your client configuration.</p>
						</div>
					{/if}
					{#if warehouses.length === 0 && !loadingWarehouses}
						<p class="mt-1 text-sm text-error-600">
							No warehouses found. <a href="/warehouses/new" class="underline">Create a warehouse</a> first.
						</p>
					{/if}
				</div>

				<div>
					<Input
						label="Storage Location"
						bind:value={storageLocation}
						placeholder="s3://bucket/path/to/catalog"
						required
						disabled={loading}
					/>
					<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
						Base path for this catalog. All tables will constitute subdirectories here.
					</p>
				</div>
			</div>

			<div class="flex justify-end gap-3 pt-6 border-t border-gray-200 dark:border-gray-700">
				<Button
					variant="secondary"
					type="button"
					disabled={loading}
					on:click={() => goto('/catalogs')}
				>
					Cancel
				</Button>
				<Button
					variant="primary"
					type="submit"
					{loading}
					disabled={loading || !name || !storageLocation}
				>
					{loading ? 'Creating...' : 'Create Catalog'}
				</Button>
			</div>
		</form>
	</Card>
</div>
