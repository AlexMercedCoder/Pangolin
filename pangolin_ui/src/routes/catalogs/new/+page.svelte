<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { catalogsApi, type CreateCatalogRequest, type FederatedAuthType } from '$lib/api/catalogs';
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

	// Form State
	let name = '';
	let isFederated = false;
	
	// Local Catalog State
	let warehouseName = '';
	let storageLocation = '';

	// Federated Catalog State
	let fedBaseUrl = '';
	let fedAuthType: FederatedAuthType = 'None';
	let fedUsername = '';
	let fedPassword = '';
	let fedToken = '';
	let fedApiKey = '';
	let fedTimeout = 30;

	const authTypeOptions = [
		{ value: 'None', label: 'None' },
		{ value: 'BasicAuth', label: 'Basic Auth' },
		{ value: 'BearerToken', label: 'Bearer Token' },
		{ value: 'ApiKey', label: 'API Key' }
	];

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
		if (selected) {
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
			let request: CreateCatalogRequest;

			if (isFederated) {
				const credentials: any = {};
				if (fedAuthType === 'BasicAuth') {
					credentials.username = fedUsername;
					credentials.password = fedPassword;
				} else if (fedAuthType === 'BearerToken') {
					credentials.token = fedToken;
				} else if (fedAuthType === 'ApiKey') {
					credentials.api_key = fedApiKey;
				}

				request = {
					name,
					catalog_type: 'Federated',
					federated_config: {
						base_url: fedBaseUrl,
						auth_type: fedAuthType,
						credentials: fedAuthType === 'None' ? undefined : credentials,
						timeout_seconds: fedTimeout
					},
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
				<Input
					label="Catalog Name"
					bind:value={name}
					placeholder="my-catalog"
					required
					disabled={loading}
				/>

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
					<div class="space-y-6 transition-all duration-300">
						<Input
							label="Base URL"
							bind:value={fedBaseUrl}
							placeholder="https://rest-catalog.example.com/api/v1"
							required
							disabled={loading}
						/>

						<Select
							label="Authentication Type"
							bind:value={fedAuthType}
							options={authTypeOptions}
							disabled={loading}
						/>

						{#if fedAuthType === 'BasicAuth'}
							<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
								<Input
									label="Username"
									bind:value={fedUsername}
									required
									disabled={loading}
								/>
								<Input
									label="Password"
									type="password"
									bind:value={fedPassword}
									required
									disabled={loading}
								/>
							</div>
						{:else if fedAuthType === 'BearerToken'}
							<Input
								label="Bearer Token"
								type="password"
								bind:value={fedToken}
								required
								disabled={loading}
							/>
						{:else if fedAuthType === 'ApiKey'}
							<Input
								label="API Key"
								type="password"
								bind:value={fedApiKey}
								required
								disabled={loading}
							/>
						{/if}
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
					disabled={loading || !name || (!isFederated && !storageLocation) || (isFederated && !fedBaseUrl)}
				>
					{loading ? 'Creating...' : 'Create Catalog'}
				</Button>
			</div>
		</form>
	</Card>
</div>
