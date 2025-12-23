<script lang="ts">
	import Modal from '$lib/components/ui/Modal.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import Select from '$lib/components/ui/Select.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import { warehousesApi, type CreateWarehouseRequest } from '$lib/api/warehouses';
    import { optimizationApi } from '$lib/api/optimization';
    import type { NameValidationResult } from '$lib/types/optimization';
	import { notifications } from '$lib/stores/notifications';

	export let open = false;
	export let onSuccess: () => void = () => {};

	let creating = false;
	let step = 1; // Multi-step form: 1=Basic, 2=Storage Type, 3=Auth, 4=Review

	// Form state
	let formData = {
		name: '',
		storageType: 's3' as 's3' | 'azure' | 'gcs',
		authMethod: 'static' as 'static' | 'iam',
		
		// S3 fields
		s3Bucket: '',
		s3Region: 'us-east-1',
		s3Endpoint: '',
		s3AccessKey: '',
		s3SecretKey: '',
		s3RoleArn: '',
		
		// Azure fields
		azureContainer: '',
		azureAccountName: '',
		azureAccountKey: '',
		azureTenantId: '',
		azureClientId: '',
		azureClientSecret: '',
		
		// GCS fields
		gcsBucket: '',
		gcsProjectId: '',
		gcsServiceAccountKey: '',
		gcsServiceAccountEmail: '',
	};

	let formErrors: Record<string, string> = {};

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
                resource_type: 'warehouse',
                names: [val]
            });
            validationResult = res.results[0];
        } catch (e) {
            console.error('Validation failed', e);
        } finally {
            validating = false;
        }
    }, 500);

    $: validateName(formData.name);

	const storageTypeOptions = [
		{ value: 's3', label: 'AWS S3 / S3-Compatible' },
		{ value: 'azure', label: 'Azure Blob Storage' },
		{ value: 'gcs', label: 'Google Cloud Storage' },
	];

	const s3RegionOptions = [
		{ value: 'us-east-1', label: 'US East (N. Virginia)' },
		{ value: 'us-east-2', label: 'US East (Ohio)' },
		{ value: 'us-west-1', label: 'US West (N. California)' },
		{ value: 'us-west-2', label: 'US West (Oregon)' },
		{ value: 'eu-west-1', label: 'EU (Ireland)' },
		{ value: 'eu-central-1', label: 'EU (Frankfurt)' },
		{ value: 'ap-southeast-1', label: 'Asia Pacific (Singapore)' },
		{ value: 'ap-northeast-1', label: 'Asia Pacific (Tokyo)' },
	];

	function validateStep(): boolean {
		formErrors = {};

        if (step === 1) {
            if (validationResult && !validationResult.available) {
                formErrors.name = validationResult.reason || 'Name is unavailable';
                return false;
            }
        }

		if (step === 1) {
			if (!formData.name.trim()) {
				formErrors.name = 'Name is required';
			} else if (!/^[a-zA-Z0-9_-]+$/.test(formData.name)) {
				formErrors.name = 'Name can only contain letters, numbers, hyphens, and underscores';
			}
		}

		if (step === 2) {
			if (formData.storageType === 's3') {
				if (!formData.s3Bucket.trim()) {
					formErrors.s3Bucket = 'Bucket name is required';
				}
			} else if (formData.storageType === 'azure') {
				if (!formData.azureContainer.trim()) {
					formErrors.azureContainer = 'Container name is required';
				}
				if (!formData.azureAccountName.trim()) {
					formErrors.azureAccountName = 'Account name is required';
				}
			} else if (formData.storageType === 'gcs') {
				if (!formData.gcsBucket.trim()) {
					formErrors.gcsBucket = 'Bucket name is required';
				}
				if (!formData.gcsProjectId.trim()) {
					formErrors.gcsProjectId = 'Project ID is required';
				}
			}
		}

		if (step === 3) {
			if (formData.authMethod === 'static') {
				if (formData.storageType === 's3') {
					if (!formData.s3AccessKey.trim()) {
						formErrors.s3AccessKey = 'Access Key is required';
					}
					if (!formData.s3SecretKey.trim()) {
						formErrors.s3SecretKey = 'Secret Key is required';
					}
				} else if (formData.storageType === 'azure') {
					if (!formData.azureAccountKey.trim()) {
						formErrors.azureAccountKey = 'Account Key is required';
					}
				} else if (formData.storageType === 'gcs') {
					if (!formData.gcsServiceAccountKey.trim()) {
						formErrors.gcsServiceAccountKey = 'Service Account Key is required';
					}
				}
			} else {
				// IAM role validation
				if (formData.storageType === 's3') {
					if (!formData.s3RoleArn.trim()) {
						formErrors.s3RoleArn = 'Role ARN is required';
					}
				} else if (formData.storageType === 'azure') {
					if (!formData.azureTenantId.trim()) {
						formErrors.azureTenantId = 'Tenant ID is required';
					}
					if (!formData.azureClientId.trim()) {
						formErrors.azureClientId = 'Client ID is required';
					}
				} else if (formData.storageType === 'gcs') {
					if (!formData.gcsServiceAccountEmail.trim()) {
						formErrors.gcsServiceAccountEmail = 'Service Account Email is required';
					}
				}
			}
		}

		return Object.keys(formErrors).length === 0;
	}

	function nextStep() {
		if (validateStep()) {
			step++;
		}
	}

	function prevStep() {
		step--;
		formErrors = {};
	}

	async function handleCreate() {
		if (!validateStep()) return;

		creating = true;

		// Build storage config based on type and auth method
		const storage_config: any = {};

		if (formData.storageType === 's3') {
			storage_config['s3.bucket'] = formData.s3Bucket;
			storage_config['s3.region'] = formData.s3Region;
			if (formData.s3Endpoint) {
				storage_config['s3.endpoint'] = formData.s3Endpoint;
			}
			if (formData.authMethod === 'static') {
				storage_config['s3.access-key-id'] = formData.s3AccessKey;
				storage_config['s3.secret-access-key'] = formData.s3SecretKey;
			}
			// Note: IAM role (role_arn) not supported in current API
		} else if (formData.storageType === 'azure') {
			storage_config['azure.container'] = formData.azureContainer;
			storage_config['adls.account-name'] = formData.azureAccountName;
			if (formData.authMethod === 'static') {
				storage_config['adls.account-key'] = formData.azureAccountKey;
			}
			// Note: OAuth fields (tenant_id, client_id, client_secret) not supported in current API
		} else if (formData.storageType === 'gcs') {
			storage_config['gcs.bucket'] = formData.gcsBucket;
			storage_config['gcs.project-id'] = formData.gcsProjectId;
			if (formData.authMethod === 'static') {
				storage_config['gcs.service-account-file'] = formData.gcsServiceAccountKey;
			}
			// Note: service_account_email not supported in current API
		}

		const request: CreateWarehouseRequest = {
			name: formData.name,
			use_sts: formData.authMethod === 'iam',
			storage_config,
		};

		try {
			await warehousesApi.create(request);
			notifications.success(`Warehouse "${formData.name}" created successfully`);
			open = false;
			resetForm();
			onSuccess();
		} catch (error: any) {
			notifications.error(`Failed to create warehouse: ${error.message || 'Unknown error'}`);
		} finally {
			creating = false;
		}
	}

	function resetForm() {
		step = 1;
		formData = {
			name: '',
			storageType: 's3',
			authMethod: 'static',
			s3Bucket: '',
			s3Region: 'us-east-1',
			s3Endpoint: '',
			s3AccessKey: '',
			s3SecretKey: '',
			s3RoleArn: '',
			azureContainer: '',
			azureAccountName: '',
			azureAccountKey: '',
			azureTenantId: '',
			azureClientId: '',
			azureClientSecret: '',
			gcsBucket: '',
			gcsProjectId: '',
			gcsServiceAccountKey: '',
			gcsServiceAccountEmail: '',
		};
		};
		formErrors = {};
        validationResult = null;
	}

	$: if (!open) {
		resetForm();
	}
</script>

<Modal bind:open size="lg">
	<div class="space-y-6">
		<!-- Header with Steps -->
		<div>
			<h2 class="text-2xl font-bold text-gray-900 dark:text-white">Create Warehouse</h2>
			<div class="mt-4 flex items-center justify-between">
				{#each [1, 2, 3, 4] as stepNum}
					<div class="flex items-center">
						<div
							class="w-8 h-8 rounded-full flex items-center justify-center text-sm font-medium
							{step >= stepNum
								? 'bg-primary-600 text-white'
								: 'bg-gray-200 dark:bg-gray-700 text-gray-600 dark:text-gray-400'}"
						>
							{stepNum}
						</div>
						{#if stepNum < 4}
							<div
								class="w-16 h-1 mx-2
								{step > stepNum ? 'bg-primary-600' : 'bg-gray-200 dark:bg-gray-700'}"
							/>
						{/if}
					</div>
				{/each}
			</div>
			<div class="mt-2 text-sm text-gray-600 dark:text-gray-400">
				{#if step === 1}Step 1: Basic Information
				{:else if step === 2}Step 2: Storage Configuration
				{:else if step === 3}Step 3: Authentication
				{:else}Step 4: Review & Create
				{/if}
			</div>
		</div>

		<!-- Step 1: Basic Info -->
		{#if step === 1}
			<div class="space-y-4">
				<div class="relative">
                    <Input
                        label="Warehouse Name"
                        bind:value={formData.name}
                        placeholder="my-warehouse"
                        required
                        error={formErrors.name}
                    />
                    <!-- Validation Indicator -->
                    <div class="absolute right-2 top-9">
                        {#if validating}
                            <svg class="animate-spin h-5 w-5 text-gray-400" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                            </svg>
                        {:else if validationResult && formData.name.length >= 2}
                            {#if validationResult.available}
                                <span class="material-icons text-green-500 text-lg">check_circle</span>
                            {:else}
                                <span class="material-icons text-red-500 text-lg" title={validationResult.reason || 'Taken'}>cancel</span>
                            {/if}
                        {/if}
                    </div>
                </div>
				<div class="p-4 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
					<p class="text-sm text-blue-800 dark:text-blue-200">
						<strong>💡 Tip:</strong> Choose a descriptive name like "production-s3" or "dev-minio". This will be referenced when creating catalogs.
					</p>
				</div>
			</div>
		{/if}

		<!-- Step 2: Storage Type & Config -->
		{#if step === 2}
			<div class="space-y-4">
				<Select
					label="Storage Type"
					bind:value={formData.storageType}
					options={storageTypeOptions}
					required
				/>

				{#if formData.storageType === 's3'}
					<Input
						label="S3 Bucket (for authentication)"
						bind:value={formData.s3Bucket}
						placeholder="my-data-bucket"
						required
						error={formErrors.s3Bucket}
					/>
					<Select
						label="AWS Region"
						bind:value={formData.s3Region}
						options={s3RegionOptions}
						required
					/>
					<Input
						label="Custom Endpoint (Optional)"
						bind:value={formData.s3Endpoint}
						placeholder="http://minio:9000 or https://s3.custom-domain.com"
						error={formErrors.s3Endpoint}
					/>
					<div class="p-4 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
						<p class="text-sm text-blue-800 dark:text-blue-200">
							<strong>💡 About Bucket:</strong> The bucket specified here is used for authentication and as a default. Each catalog can specify its own storage location (same bucket or different) when created.
						</p>
					</div>
					<div class="p-4 bg-yellow-50 dark:bg-yellow-900/20 rounded-lg">
						<p class="text-sm text-yellow-800 dark:text-yellow-200">
							<strong>🔧 S3-Compatible Storage:</strong> If using MinIO, LocalStack, or other S3-compatible storage, provide the endpoint URL above. Leave blank for AWS S3.
						</p>
					</div>
				{:else if formData.storageType === 'azure'}
					<Input
						label="Container Name (for authentication)"
						bind:value={formData.azureContainer}
						placeholder="data-container"
						required
						error={formErrors.azureContainer}
					/>
					<Input
						label="Storage Account Name"
						bind:value={formData.azureAccountName}
						placeholder="mystorageaccount"
						required
						error={formErrors.azureAccountName}
					/>
				{:else if formData.storageType === 'gcs'}
					<Input
						label="GCS Bucket (for authentication)"
						bind:value={formData.gcsBucket}
						placeholder="my-gcs-bucket"
						required
						error={formErrors.gcsBucket}
					/>
					<Input
						label="Project ID"
						bind:value={formData.gcsProjectId}
						placeholder="my-project-123456"
						required
						error={formErrors.gcsProjectId}
					/>
				{/if}
			</div>
		{/if}

		<!-- Step 3: Authentication -->
		{#if step === 3}
			<div class="space-y-4">
				<div>
					<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">
						Authentication Method <span class="text-error-600">*</span>
					</label>
					<div class="grid grid-cols-2 gap-4">
						<button
							type="button"
							on:click={() => (formData.authMethod = 'static')}
							class="p-4 border-2 rounded-lg text-left transition-all
								{formData.authMethod === 'static'
									? 'border-primary-600 bg-primary-50 dark:bg-primary-900/20'
									: 'border-gray-300 dark:border-gray-600 hover:border-gray-400'}"
						>
							<div class="font-medium text-gray-900 dark:text-white">Static Credentials</div>
							<div class="mt-1 text-sm text-gray-600 dark:text-gray-400">
								Provide access keys directly
							</div>
							<div class="mt-2 text-xs text-gray-500 dark:text-gray-500">
								✅ Simple setup<br />
								⚠️ Less secure<br />
								📝 For dev/testing
							</div>
						</button>
						<button
							type="button"
							on:click={() => (formData.authMethod = 'iam')}
							class="p-4 border-2 rounded-lg text-left transition-all
								{formData.authMethod === 'iam'
									? 'border-primary-600 bg-primary-50 dark:bg-primary-900/20'
									: 'border-gray-300 dark:border-gray-600 hover:border-gray-400'}"
						>
							<div class="font-medium text-gray-900 dark:text-white">IAM Role / OAuth</div>
							<div class="mt-1 text-sm text-gray-600 dark:text-gray-400">
								Use temporary credentials
							</div>
							<div class="mt-2 text-xs text-gray-500 dark:text-gray-500">
								✅ More secure<br />
								✅ Auto-rotation<br />
								🏢 For production
							</div>
						</button>
					</div>
				</div>

				{#if formData.authMethod === 'static'}
					<!-- Static Credentials -->
					{#if formData.storageType === 's3'}
						<Input
							label="Access Key ID"
							bind:value={formData.s3AccessKey}
							placeholder="AKIAIOSFODNN7EXAMPLE"
							required
							error={formErrors.s3AccessKey}
						/>
						<Input
							label="Secret Access Key"
							type="password"
							bind:value={formData.s3SecretKey}
							placeholder="wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"
							required
							error={formErrors.s3SecretKey}
						/>
					{:else if formData.storageType === 'azure'}
						<Input
							label="Account Key"
							type="password"
							bind:value={formData.azureAccountKey}
							placeholder="Storage account access key"
							required
							error={formErrors.azureAccountKey}
						/>
					{:else if formData.storageType === 'gcs'}
						<div>
							<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
								Service Account Key (JSON) <span class="text-error-600">*</span>
							</label>
							<textarea
								bind:value={formData.gcsServiceAccountKey}
								placeholder={`{"type": "service_account", "project_id": "...", ...}`}
								rows="6"
								class="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white font-mono text-sm"
							/>
							{#if formErrors.gcsServiceAccountKey}
								<p class="mt-1 text-sm text-error-600">{formErrors.gcsServiceAccountKey}</p>
							{/if}
						</div>
					{/if}
					<div class="p-4 bg-yellow-50 dark:bg-yellow-900/20 rounded-lg">
						<p class="text-sm text-yellow-800 dark:text-yellow-200">
							<strong>⚠️ Security Note:</strong> Static credentials are stored in Pangolin and passed to clients. Use IAM roles for production environments.
						</p>
					</div>
				{:else}
					<!-- IAM Role / OAuth -->
					{#if formData.storageType === 's3'}
						<Input
							label="IAM Role ARN"
							bind:value={formData.s3RoleArn}
							placeholder="arn:aws:iam::123456789012:role/PangolinDataAccess"
							required
							error={formErrors.s3RoleArn}
						/>
						<div class="p-4 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
							<p class="text-sm text-blue-800 dark:text-blue-200">
								<strong>📚 Setup Guide:</strong> See <a href="/docs/features/iam_roles.md" class="underline">IAM Roles Documentation</a> for creating and configuring the role with proper trust policies.
							</p>
						</div>
					{:else if formData.storageType === 'azure'}
						<Input
							label="Azure AD Tenant ID"
							bind:value={formData.azureTenantId}
							placeholder="00000000-0000-0000-0000-000000000000"
							required
							error={formErrors.azureTenantId}
						/>
						<Input
							label="Managed Identity / Service Principal Client ID"
							bind:value={formData.azureClientId}
							placeholder="00000000-0000-0000-0000-000000000000"
							required
							error={formErrors.azureClientId}
						/>
						<Input
							label="Client Secret (Optional for Managed Identity)"
							type="password"
							bind:value={formData.azureClientSecret}
							placeholder="Leave blank if using Managed Identity"
							error={formErrors.azureClientSecret}
						/>
						<div class="p-4 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
							<p class="text-sm text-blue-800 dark:text-blue-200">
								<strong>📚 Setup Guide:</strong> Grant "Storage Blob Data Contributor" role to the managed identity or service principal.
							</p>
						</div>
					{:else if formData.storageType === 'gcs'}
						<Input
							label="Service Account Email"
							bind:value={formData.gcsServiceAccountEmail}
							placeholder="pangolin-data@project-id.iam.gserviceaccount.com"
							required
							error={formErrors.gcsServiceAccountEmail}
						/>
						<div class="p-4 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
							<p class="text-sm text-blue-800 dark:text-blue-200">
								<strong>📚 Setup Guide:</strong> Pangolin will impersonate this service account. Ensure Pangolin's service account has "Service Account Token Creator" role.
							</p>
						</div>
					{/if}
					<div class="p-4 bg-green-50 dark:bg-green-900/20 rounded-lg">
						<p class="text-sm text-green-800 dark:text-green-200">
							<strong>✅ Recommended:</strong> IAM roles provide temporary credentials that auto-expire, eliminating the need to store long-lived secrets.
						</p>
					</div>
				{/if}
			</div>
		{/if}

		<!-- Step 4: Review -->
		{#if step === 4}
			<div class="space-y-4">
				<div class="p-6 bg-gray-50 dark:bg-gray-800 rounded-lg space-y-4">
					<div>
						<div class="text-sm text-gray-500 dark:text-gray-400">Warehouse Name</div>
						<div class="text-lg font-medium text-gray-900 dark:text-white">{formData.name}</div>
					</div>
					<div>
						<div class="text-sm text-gray-500 dark:text-gray-400">Storage Type</div>
						<div class="text-lg font-medium text-gray-900 dark:text-white">
							{storageTypeOptions.find((o) => o.value === formData.storageType)?.label}
						</div>
					</div>
					<div>
						<div class="text-sm text-gray-500 dark:text-gray-400">Authentication</div>
						<div class="text-lg font-medium text-gray-900 dark:text-white">
							{formData.authMethod === 'static' ? 'Static Credentials' : 'IAM Role / OAuth'}
						</div>
					</div>
					{#if formData.storageType === 's3'}
						<div>
							<div class="text-sm text-gray-500 dark:text-gray-400">Bucket</div>
							<div class="text-lg font-medium text-gray-900 dark:text-white">{formData.s3Bucket}</div>
						</div>
						<div>
							<div class="text-sm text-gray-500 dark:text-gray-400">Region</div>
							<div class="text-lg font-medium text-gray-900 dark:text-white">{formData.s3Region}</div>
						</div>
						{#if formData.s3Endpoint}
							<div>
								<div class="text-sm text-gray-500 dark:text-gray-400">Custom Endpoint</div>
								<div class="text-lg font-medium text-gray-900 dark:text-white">{formData.s3Endpoint}</div>
							</div>
						{/if}
					{/if}
				</div>
				<div class="p-4 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
					<p class="text-sm text-blue-800 dark:text-blue-200">
						<strong>📝 Next Steps:</strong> After creating the warehouse, you can link catalogs to it for credential vending.
					</p>
				</div>
			</div>
		{/if}

		<!-- Actions -->
		<div class="flex items-center justify-between pt-4 border-t border-gray-200 dark:border-gray-700">
			<div>
				{#if step > 1}
					<Button variant="secondary" on:click={prevStep} disabled={creating}>
						← Previous
					</Button>
				{/if}
			</div>
			<div class="flex items-center gap-3">
				<Button variant="secondary" on:click={() => (open = false)} disabled={creating}>
					Cancel
				</Button>
				{#if step < 4}
					<Button on:click={nextStep}>
						Next →
					</Button>
				{:else}
					<Button on:click={handleCreate} disabled={creating}>
						{creating ? 'Creating...' : 'Create Warehouse'}
					</Button>
				{/if}
			</div>
		</div>
	</div>
</Modal>
