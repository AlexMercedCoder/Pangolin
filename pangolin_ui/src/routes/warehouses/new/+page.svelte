<script lang="ts">
	import { goto } from '$app/navigation';
	import { warehousesApi, type CreateWarehouseRequest } from '$lib/api/warehouses';
	import Button from '$lib/components/ui/Button.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import Card from '$lib/components/ui/Card.svelte';
	import { notifications } from '$lib/stores/notifications';

	let loading = false;
	let error = '';

	let name = '';
	let use_sts = false;
	let storageType: 's3' | 'azure' | 'gcs' = 's3';

	// S3 Fields
	let bucket = '';
	let region = 'us-east-1';
	let endpoint = '';
	let pathStyle = false;
	let accessKeyId = '';
	let secretAccessKey = '';
	let roleArn = '';
	let externalId = '';

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

	async function handleSubmit() {
		loading = true;
		error = '';

		try {
			const request: CreateWarehouseRequest = {
				name,
				use_sts,
				storage_config: {
					type: storageType,
				}
			};

			if (storageType === 's3') {
				request.storage_config.bucket = bucket;
				request.storage_config.region = region;
				if (endpoint) request.storage_config.endpoint = endpoint;
				if (pathStyle) request.storage_config['s3.path-style-access'] = 'true';
				
				if (use_sts) {
					// Add IAM role configuration for STS
					if (roleArn) request.storage_config.role_arn = roleArn;
					if (externalId) request.storage_config.external_id = externalId;
				} else {
					// Add static credentials if not using STS
					if (accessKeyId && secretAccessKey) {
						request.storage_config.access_key_id = accessKeyId;
						request.storage_config.secret_access_key = secretAccessKey;
					}
				}
			} else if (storageType === 'azure') {
				request.storage_config.account_name = accountName;
				request.storage_config.container = container;
				if (use_sts) {
					request.storage_config.tenant_id = tenantId;
					request.storage_config.client_id = clientId;
					request.storage_config.client_secret = clientSecret;
				} else {
					request.storage_config.account_key = accountKey;
				}
			} else if (storageType === 'gcs') {
				request.storage_config.bucket = bucket;
				if (projectId) request.storage_config.project_id = projectId;
				if (serviceAccountJson) request.storage_config.service_account_json = serviceAccountJson;
			}

			await warehousesApi.create(request);
			notifications.success('Warehouse created successfully');
			goto('/warehouses');
		} catch (e: any) {
			error = e.message || 'Failed to create warehouse';
			notifications.error(error);
		} finally {
			loading = false;
		}
	}
</script>

<svelte:head>
	<title>Create Warehouse - Pangolin</title>
</svelte:head>

<div class="max-w-3xl mx-auto space-y-6">
	<div>
		<h1 class="text-3xl font-bold text-gray-900 dark:text-white">Create Warehouse</h1>
		<p class="mt-2 text-gray-600 dark:text-gray-400">
			Configure a new storage location for your data
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
				<!-- Basic Info -->
				<Input
					label="Name"
					bind:value={name}
					placeholder="my-warehouse"
					required
					disabled={loading}
				/>

				<div class="space-y-2">
					<label for="storage_type" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
						Storage Type
					</label>
					<select
						id="storage_type"
						bind:value={storageType}
						class="block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg shadow-sm focus:outline-none focus:ring-2 focus:ring-primary-500 bg-white dark:bg-gray-800 dark:text-white"
						disabled={loading}
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
						disabled={loading}
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
								disabled={loading}
							/>
							<Input
								label="Region"
								bind:value={region}
								placeholder="us-east-1"
								required
								disabled={loading}
							/>
							<Input
								label="Endpoint (Optional)"
								bind:value={endpoint}
								placeholder="http://localhost:9000"
								helpText="For MinIO or custom S3-compatible endpoints"
								disabled={loading}
							/>

							<div class="flex items-center gap-2">
								<input
									type="checkbox"
									id="path_style"
									bind:checked={pathStyle}
									disabled={loading}
									class="rounded border-gray-300 text-primary-600 focus:ring-primary-500"
								/>
								<label for="path_style" class="text-sm font-medium text-gray-700 dark:text-gray-300">
									Use Path-Style Access (Required for MinIO)
								</label>
							</div>
							
							{#if use_sts}
								<div class="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-4">
									<p class="text-sm text-blue-800 dark:text-blue-200">
										<strong>IAM Role (STS):</strong> Provide the ARN of the IAM role to assume for S3 access.
									</p>
								</div>
								<Input
									label="Role ARN"
									bind:value={roleArn}
									placeholder="arn:aws:iam::123456789012:role/MyS3AccessRole"
									helpText="IAM Role ARN to assume for accessing S3"
									required
									disabled={loading}
								/>
								<Input
									label="External ID (Optional)"
									bind:value={externalId}
									placeholder="unique-external-id"
									helpText="External ID for additional security when assuming the role"
									disabled={loading}
								/>
							{:else}
								<div class="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-4">
									<p class="text-sm text-yellow-800 dark:text-yellow-200">
										<strong>Static Credentials:</strong> Provide AWS access credentials. For production, use STS/IAM roles instead.
									</p>
								</div>
								<Input
									label="Access Key ID"
									bind:value={accessKeyId}
									placeholder="AKIAIOSFODNN7EXAMPLE"
									helpText="AWS Access Key ID or MinIO access key"
									disabled={loading}
								/>
								<Input
									label="Secret Access Key"
									type="password"
									bind:value={secretAccessKey}
									placeholder="wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"
									helpText="AWS Secret Access Key or MinIO secret key"
									disabled={loading}
								/>
							{/if}
						</div>
					{:else if storageType === 'azure'}
						<div class="grid gap-4">
							<Input
								label="Storage Account Name"
								bind:value={accountName}
								required
								disabled={loading}
							/>
							<Input
								label="Container Name"
								bind:value={container}
								required
								disabled={loading}
							/>
							
							{#if use_sts}
								<div class="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-4">
									<p class="text-sm text-blue-800 dark:text-blue-200">
										<strong>Azure AD (Service Principal):</strong> Provide Azure AD credentials for authentication.
									</p>
								</div>
								<Input
									label="Tenant ID"
									bind:value={tenantId}
									placeholder="00000000-0000-0000-0000-000000000000"
									helpText="Azure AD Tenant ID"
									required
									disabled={loading}
								/>
								<Input
									label="Client ID"
									bind:value={clientId}
									placeholder="00000000-0000-0000-0000-000000000000"
									helpText="Azure AD Application (Client) ID"
									required
									disabled={loading}
								/>
								<Input
									label="Client Secret"
									type="password"
									bind:value={clientSecret}
									helpText="Azure AD Application Client Secret"
									required
									disabled={loading}
								/>
							{:else}
								<div class="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-4">
									<p class="text-sm text-yellow-800 dark:text-yellow-200">
										<strong>Account Key:</strong> Provide the storage account key. For production, use Azure AD authentication instead.
									</p>
								</div>
								<Input
									label="Account Key"
									type="password"
									bind:value={accountKey}
									helpText="Azure Storage Account Key"
									required
									disabled={loading}
								/>
							{/if}
						</div>
					{:else if storageType === 'gcs'}
						<div class="grid gap-4">
							<div class="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-4">
								<p class="text-sm text-blue-800 dark:text-blue-200">
									<strong>Service Account:</strong> Provide your GCP service account credentials in JSON format.
								</p>
							</div>
							
							<Input
								label="Project ID"
								bind:value={projectId}
								placeholder="my-gcp-project"
								helpText="Google Cloud Project ID"
								required
								disabled={loading}
							/>
							
							<Input
								label="Bucket Name"
								bind:value={bucket}
								placeholder="my-gcs-bucket"
								required
								disabled={loading}
							/>
							
							<div>
								<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
									Service Account JSON (Optional)
								</label>
								<textarea
									bind:value={serviceAccountJson}
									rows="5"
									placeholder="Paste service account JSON here..."
									class="block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg shadow-sm focus:outline-none focus:ring-2 focus:ring-primary-500 bg-white dark:bg-gray-800 dark:text-white font-mono text-sm"
									disabled={loading}
								></textarea>
								<p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
									Paste the entire service account JSON key file. Leave empty to use Application Default Credentials.
								</p>
							</div>
						</div>
					{/if}
				</div>
			</div>

			<div class="flex justify-end gap-3 pt-6 border-t border-gray-200 dark:border-gray-700">
				<Button
					variant="secondary"
					type="button"
					disabled={loading}
					on:click={() => goto('/warehouses')}
				>
					Cancel
				</Button>
				<Button
					variant="primary"
					type="submit"
					{loading}
					disabled={loading || !name}
				>
					{loading ? 'Creating...' : 'Create Warehouse'}
				</Button>
			</div>
		</form>
	</Card>
</div>
