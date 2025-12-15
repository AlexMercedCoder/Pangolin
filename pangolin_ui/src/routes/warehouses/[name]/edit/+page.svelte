<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { warehousesApi, type Warehouse, type UpdateWarehouseRequest } from '$lib/api/warehouses';
	import Button from '$lib/components/ui/Button.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import Card from '$lib/components/ui/Card.svelte';
	import { notifications } from '$lib/stores/notifications';

	let warehouse: Warehouse | null = null;
	let loading = true;
	let submitting = false;
	let error = '';

	let use_sts = false;
	let storageType: 's3' | 'azure' | 'gcs' = 's3';

	// S3 Fields
	let bucket = '';
	let region = 'us-east-1';
	let endpoint = '';

	// Azure Fields
	let accountName = '';
	let accountKey = '';
	let container = '';
	let tenantId = '';
	let clientId = '';
	let clientSecret = '';

	// GCS Fields
	let projectId = '';
	let serviceAccountJson = '';

	$: warehouseName = $page.params.name;

	onMount(async () => {
		await loadWarehouse();
	});

	async function loadWarehouse() {
		if (!warehouseName) return;

		loading = true;
		try {
			warehouse = await warehousesApi.get(warehouseName);
			
			// Pre-populate form
			use_sts = warehouse.use_sts;
			storageType = warehouse.storage_config.type;

			if (storageType === 's3') {
				bucket = warehouse.storage_config.bucket || '';
				region = warehouse.storage_config.region || 'us-east-1';
				endpoint = warehouse.storage_config.endpoint || '';
			} else if (storageType === 'azure') {
				accountName = warehouse.storage_config.account_name || '';
				container = warehouse.storage_config.container || '';
				accountKey = warehouse.storage_config.account_key || '';
				tenantId = warehouse.storage_config.tenant_id || '';
				clientId = warehouse.storage_config.client_id || '';
				clientSecret = warehouse.storage_config.client_secret || '';
			} else if (storageType === 'gcs') {
				bucket = warehouse.storage_config.bucket || '';
				serviceAccountJson = warehouse.storage_config.service_account_json || '';
			}
		} catch (e: any) {
			error = e.message || 'Failed to load warehouse';
			notifications.error(error);
			goto('/warehouses');
		}
		loading = false;
	}

	async function handleSubmit() {
		submitting = true;
		error = '';

		try {
			const request: UpdateWarehouseRequest = {
				use_sts,
				storage_config: {
					type: storageType,
				}
			};

			if (storageType === 's3') {
				request.storage_config!.bucket = bucket;
				request.storage_config!.region = region;
				if (endpoint) request.storage_config!.endpoint = endpoint;
			} else if (storageType === 'azure') {
				request.storage_config!.account_name = accountName;
				request.storage_config!.container = container;
				if (use_sts) {
					request.storage_config!.tenant_id = tenantId;
					request.storage_config!.client_id = clientId;
					request.storage_config!.client_secret = clientSecret;
				} else {
					request.storage_config!.account_key = accountKey;
				}
			} else if (storageType === 'gcs') {
				request.storage_config!.bucket = bucket;
				if (serviceAccountJson) request.storage_config!.service_account_json = serviceAccountJson;
			}

			await warehousesApi.update(warehouseName, request);
			notifications.success(`Warehouse "${warehouseName}" updated successfully`);
			goto(`/warehouses/${encodeURIComponent(warehouseName)}`);
		} catch (e: any) {
			error = e.message || 'Failed to update warehouse';
			notifications.error(error);
		} finally {
			submitting = false;
		}
	}
</script>

<svelte:head>
	<title>Edit {warehouse?.name || 'Warehouse'} - Pangolin</title>
</svelte:head>

<div class="max-w-3xl mx-auto space-y-6">
	<div>
		<div class="flex items-center gap-3">
			<button
				on:click={() => goto(`/warehouses/${encodeURIComponent(warehouseName)}`)}
				class="text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white"
			>
				← Back
			</button>
			<h1 class="text-3xl font-bold text-gray-900 dark:text-white">
				Edit Warehouse: {warehouse?.name || 'Loading...'}
			</h1>
		</div>
		<p class="mt-2 text-gray-600 dark:text-gray-400">
			Update warehouse storage configuration
		</p>
	</div>

	{#if loading}
		<Card>
			<div class="flex items-center justify-center py-12">
				<div class="w-8 h-8 border-3 border-primary-600 border-t-transparent rounded-full animate-spin" />
			</div>
		</Card>
	{:else if warehouse}
		<Card>
			<form on:submit|preventDefault={handleSubmit} class="space-y-6">
				{#if error}
					<div class="p-4 bg-error-50 dark:bg-error-900/20 text-error-700 dark:text-error-200 rounded-lg">
						{error}
					</div>
				{/if}

				<div class="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-4">
					<p class="text-sm text-blue-800 dark:text-blue-200">
						<strong>Note:</strong> Warehouse name cannot be changed. To rename, create a new warehouse.
					</p>
				</div>

				<div class="grid gap-6">
					<Input
						label="Name"
						value={warehouse.name}
						disabled
						helpText="Warehouse name is immutable"
					/>

					<div class="space-y-2">
						<label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
							Storage Type
						</label>
						<select
							bind:value={storageType}
							class="block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg shadow-sm focus:outline-none focus:ring-2 focus:ring-primary-500 bg-white dark:bg-gray-800 dark:text-white"
							disabled={submitting}
						>
							<option value="s3">Amazon S3 / MinIO</option>
							<option value="azure">Azure Blob Storage</option>
							<option value="gcs">Google Cloud Storage</option>
						</select>
					</div>

					<div class="flex items-center gap-2">
						<input
							type="checkbox"
							id="use_sts"
							bind:checked={use_sts}
							disabled={submitting}
							class="rounded border-gray-300 text-primary-600 focus:ring-primary-500"
						/>
						<label for="use_sts" class="text-sm font-medium text-gray-700 dark:text-gray-300">
							Use STS (Security Token Service) / IAM Roles
						</label>
					</div>

					<div class="border-t border-gray-200 dark:border-gray-700 pt-6">
						<h3 class="text-lg font-medium text-gray-900 dark:text-white mb-4">
							Storage Configuration
						</h3>

						{#if storageType === 's3'}
							<div class="grid gap-4">
								<Input
									label="Bucket Name"
									bind:value={bucket}
									placeholder="my-bucket"
									required
									disabled={submitting}
								/>
								<Input
									label="Region"
									bind:value={region}
									placeholder="us-east-1"
									required
									disabled={submitting}
								/>
								<Input
									label="Endpoint (Optional)"
									bind:value={endpoint}
									placeholder="http://localhost:9000"
									disabled={submitting}
								/>
							</div>
						{:else if storageType === 'azure'}
							<div class="grid gap-4">
								<Input
									label="Storage Account Name"
									bind:value={accountName}
									required
									disabled={submitting}
								/>
								<Input
									label="Container Name"
									bind:value={container}
									required
									disabled={submitting}
								/>
								{#if use_sts}
									<Input
										label="Tenant ID"
										bind:value={tenantId}
										required
										disabled={submitting}
									/>
									<Input
										label="Client ID"
										bind:value={clientId}
										required
										disabled={submitting}
									/>
									<Input
										label="Client Secret"
										type="password"
										bind:value={clientSecret}
										required
										disabled={submitting}
									/>
								{:else}
									<Input
										label="Account Key"
										type="password"
										bind:value={accountKey}
										required
										disabled={submitting}
									/>
								{/if}
							</div>
						{:else if storageType === 'gcs'}
							<div class="grid gap-4">
								<Input
									label="Bucket Name"
									bind:value={bucket}
									required
									disabled={submitting}
								/>
								<div>
									<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
										Service Account JSON
									</label>
									<textarea
										bind:value={serviceAccountJson}
										rows="5"
										class="block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg shadow-sm focus:outline-none focus:ring-2 focus:ring-primary-500 bg-white dark:bg-gray-800 dark:text-white font-mono text-sm"
										disabled={submitting}
									></textarea>
								</div>
							</div>
						{/if}
					</div>
				</div>

				<div class="flex justify-end gap-3 pt-6 border-t border-gray-200 dark:border-gray-700">
					<Button
						variant="secondary"
						type="button"
						disabled={submitting}
						on:click={() => goto(`/warehouses/${encodeURIComponent(warehouseName)}`)}
					>
						Cancel
					</Button>
					<Button
						variant="primary"
						type="submit"
						loading={submitting}
						disabled={submitting}
					>
						{submitting ? 'Updating...' : 'Update Warehouse'}
					</Button>
				</div>
			</form>
		</Card>
	{/if}
</div>
