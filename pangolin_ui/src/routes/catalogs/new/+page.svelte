<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { catalogsApi, type CreateCatalogRequest } from '$lib/api/catalogs';
	import { warehousesApi, type Warehouse } from '$lib/api/warehouses';
	import Button from '$lib/components/ui/Button.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import Select from '$lib/components/ui/Select.svelte';
	import Card from '$lib/components/ui/Card.svelte';
    import { optimizationApi } from '$lib/api/optimization';
    import type { NameValidationResult } from '$lib/types/optimization';
	import { notifications } from '$lib/stores/notifications';

	let loading = false;
	let error = '';
	
	let warehouses: { value: string; label: string; full: Warehouse }[] = [];
	let loadingWarehouses = true;

	// Form State
	let name = '';
	let isFederated = false;
	
	// Local Catalog State
	let warehouseName = '';
	let storageLocation = '';

	// Federated Catalog State
	let properties: { key: string; value: string }[] = [{ key: 'uri', value: '' }];

     // Validation
    let validationResult: NameValidationResult | null = null;
    let validating = false;
    let debounceTimer: any;

    function debounce(func: Function, wait: number) {
        return (...args: any[]) => {
            clearTimeout(debounceTimer);
            debounceTimer = setTimeout(() => func(...args), wait);
        };
    }

    const validateName = debounce(async (val: string) => {
        if (!val || val.length < 2) {
            validationResult = null;
            return;
        }

        validating = true;
        try {
            const res = await optimizationApi.validateNames({
                resource_type: 'catalog',
                names: [val]
            });
            validationResult = res.results[0];
        } catch (e) {
            console.error('Validation failed', e);
        } finally {
            validating = false;
        }
    }, 500);

    $: validateName(name);

	function addProperty() {
		properties = [...properties, { key: '', value: '' }];
	}

	function removeProperty(index: number) {
		properties = properties.filter((_, i) => i !== index);
	}

	onMount(async () => {
		try {
			const wList = await warehousesApi.list();
			warehouses = wList.map(w => ({ 
				value: w.name, 
				label: w.name,
				full: w
			}));
		} catch (e: any) {
			console.error('Error fetching warehouses:', e);
			notifications.error('Failed to load warehouses');
		} finally {
			loadingWarehouses = false;
		}
	});

	// Auto-fill storage location logic (only for Local)
	$: if (!isFederated && warehouseName && !storageLocation) {
		const selected = warehouses.find(w => w.value === warehouseName);
		if (selected?.full?.storage_config) {
			const w = selected.full;
			const bucket = w.storage_config?.['s3.bucket'] || w.storage_config?.['azure.container'] || w.storage_config?.['gcs.bucket'] || 'bucket';
			const type = w.storage_config?.['s3.bucket'] ? 's3' 
			           : w.storage_config?.['adls.account-name'] ? 'azure'
			           : w.storage_config?.['gcs.bucket'] ? 'gcs'
			           : 's3';
			
			if (type === 'azure') {
				storageLocation = `abfss://${bucket}@${w.storage_config?.['adls.account-name']}.dfs.core.windows.net/${name || 'catalog'}`;
			} else if (type === 's3') {
				storageLocation = `s3://${bucket}/${name || 'catalog'}`;
			} else if (type === 'gcs') {
				storageLocation = `gs://${bucket}/${name || 'catalog'}`;
			}
		}
	}

	async function handleSubmit() {
        if (validationResult && !validationResult.available) {
             notifications.error(validationResult.reason || 'Catalog name is unavailable');
             return;
        }

		loading = true;
		error = '';

		try {
			let request: CreateCatalogRequest;

			if (isFederated) {
				const propsMap: Record<string, string> = {};
				for (const p of properties) {
					if (p.key.trim()) {
						propsMap[p.key.trim()] = p.value;
					}
				}

				request = {
					name,
					catalog_type: 'Federated',
					federated_config: { properties: propsMap },
					properties: {}
				};
			} else {
				request = {
					name,
					catalog_type: 'Local',
					warehouse_name: warehouseName || undefined,
					storage_location: storageLocation,
					properties: {}
				};
			}

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
			Create a new Iceberg catalog. Choose between a standard managed catalog or federating an external one.
		</p>
	</div>

	<Card>
		<form on:submit|preventDefault={handleSubmit} class="space-y-6">
			{#if error}
				<div class="p-4 bg-error-50 dark:bg-error-900/20 text-error-700 dark:text-error-200 rounded-lg">
					{error}
				</div>
			{/if}

			<div class="space-y-6">
			<div class="space-y-6">
                <div class="relative">
    				<Input
	    				label="Catalog Name"
		    			bind:value={name}
			    		placeholder="my-catalog"
				    	required
					    disabled={loading}
    				/>
                    <!-- Validation Indicator -->
                    <div class="absolute right-2 top-9">
                        {#if validating}
                            <svg class="animate-spin h-5 w-5 text-gray-400" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                            </svg>
                        {:else if validationResult && name.length >= 2}
                            {#if validationResult.available}
                                <span class="material-icons text-green-500 text-lg">check_circle</span>
                            {:else}
                                <span class="material-icons text-red-500 text-lg" title={validationResult.reason || 'Taken'}>cancel</span>
                            {/if}
                        {/if}
                    </div>
                     {#if validationResult && !validationResult.available}
                        <p class="text-xs text-red-500 mt-1 ml-1">{validationResult.reason || 'Name is unavailable'}</p>
                    {/if}
                </div>

				<!-- Catalog Type Toggle -->
				<div>
					<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
						Catalog Type
					</label>
					<div class="flex items-center space-x-4">
						<label class="flex items-center">
							<input 
								type="radio" 
								bind:group={isFederated} 
								value={false} 
								class="h-4 w-4 text-primary-600 focus:ring-primary-500 border-gray-300"
							>
							<span class="ml-2 text-gray-700 dark:text-gray-300">Local (Managed)</span>
						</label>
						<label class="flex items-center">
							<input 
								type="radio" 
								bind:group={isFederated} 
								value={true} 
								class="h-4 w-4 text-primary-600 focus:ring-primary-500 border-gray-300"
							>
							<span class="ml-2 text-gray-700 dark:text-gray-300">Federated (External)</span>
						</label>
					</div>
				</div>

				<hr class="border-gray-200 dark:border-gray-700" />

				{#if !isFederated}
					<!-- Local Catalog Fields -->
					<div class="space-y-6 transition-all duration-300">
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
				{:else}
					<!-- Federated Catalog Fields -->
					<!-- Federated Catalog Fields -->
					<div class="space-y-4 transition-all duration-300">
						<div class="flex items-center justify-between">
							<h3 class="text-sm font-medium text-gray-700 dark:text-gray-300">Configuration Properties</h3>
							<Button variant="secondary" size="sm" type="button" on:click={addProperty}>
								+ Add Property
							</Button>
						</div>

						<div class="space-y-3">
							{#each properties as prop, i}
								<div class="flex gap-2 items-start">
									<div class="flex-1">
										<Input
											placeholder="Key (e.g. uri)"
											bind:value={prop.key}
											disabled={loading}
										/>
									</div>
									<div class="flex-1">
										<Input
											placeholder="Value"
											bind:value={prop.value}
											disabled={loading}
										/>
									</div>
									<button 
										type="button"
										class="mt-2 text-gray-400 hover:text-red-500 transition-colors"
										on:click={() => removeProperty(i)}
										disabled={loading}
										title="Remove property"
									>
										<svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
											<path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
										</svg>
									</button>
								</div>
							{/each}
						</div>

						<div class="text-xs text-gray-500 mt-2">
							<p>Common properties:</p>
							<ul class="list-disc pl-4 mt-1 space-y-1">
								<li><code>uri</code>: The URL of the external catalog (Required)</li>
								<li><code>warehouse</code>: The warehouse name to use</li>
								<li><code>token</code>: Authentication token</li>
								<li><code>credential</code>: Client credential (id:secret)</li>
							</ul>
						</div>
					</div>
				{/if}
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
					disabled={loading || !name || (!isFederated && !storageLocation) || (isFederated && properties.every(p => !p.key || !p.value)) || (validationResult !== null && !validationResult.available)}
				>
					{loading ? 'Creating...' : 'Create Catalog'}
				</Button>
			</div>
		</form>
	</Card>
</div>
