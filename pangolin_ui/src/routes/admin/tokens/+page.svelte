<script lang="ts">
	import { onMount } from 'svelte';
	import { authApi, type GenerateTokenRequest } from '$lib/api/auth';
	import { tenantsApi, type Tenant } from '$lib/api/tenants';
	import { notifications } from '$lib/stores/notifications';
	import Card from '$lib/components/ui/Card.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import Select from '$lib/components/ui/Select.svelte';
	import Textarea from '$lib/components/ui/Textarea.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';

	let tenants: Tenant[] = [];
	let loadingTenants = false;

	// Generate Token Form
	let generateForm: GenerateTokenRequest = {
		tenant_id: '',
		username: '',
		roles: [],
		expires_in_hours: 24
	};
	let expiryOptions = [
		{ value: 1, label: '1 Hour' },
		{ value: 24, label: '24 Hours' },
		{ value: 168, label: '7 Days' },
		{ value: 720, label: '30 Days' },
		{ value: 8760, label: '1 Year' }
	];
	let selectedExpiry = 24;
	let rolesInput = '';
	let generating = false;
	let generatedToken: string | null = null;
	let showTokenModal = false;

	// Revoke Token Form
	let revokeTokenId = '';
	let revokeReason = '';
	let revoking = false;

	// Cleanup Form
	let cleaning = false;

	onMount(async () => {
		await loadTenants();
	});

	async function loadTenants() {
		loadingTenants = true;
		try {
			tenants = await tenantsApi.list();
			if (tenants.length > 0) {
				generateForm.tenant_id = tenants[0].id;
			}
		} catch (error: any) {
			notifications.error(`Failed to load tenants: ${error.message}`);
		} finally {
			loadingTenants = false;
		}
	}

	async function handleGenerate() {
		if (!generateForm.tenant_id) {
			notifications.error('Tenant is required');
			return;
		}

		generating = true;
		try {
			const roles = rolesInput.split(',').map(r => r.trim()).filter(r => r.length > 0);
			
			const response = await authApi.generateToken({
				tenant_id: generateForm.tenant_id,
				username: generateForm.username || undefined,
				roles: roles.length > 0 ? roles : undefined,
				expires_in_hours: selectedExpiry
			});

			generatedToken = response.token;
			showTokenModal = true;
			notifications.success('Token generated successfully');
		} catch (error: any) {
			notifications.error(`Failed to generate token: ${error.message}`);
		} finally {
			generating = false;
		}
	}

	function closeTokenModal() {
		showTokenModal = false;
		generatedToken = null;
	}

	async function copyToken() {
		if (generatedToken) {
			try {
				await navigator.clipboard.writeText(generatedToken);
				notifications.success('Token copied to clipboard');
			} catch (err) {
				notifications.error('Failed to copy to clipboard');
			}
		}
	}

	async function handleRevoke() {
		if (!revokeTokenId) {
			notifications.error('Token ID is required');
			return;
		}

		if (!confirm('Are you sure you want to revoke this token? This action cannot be undone.')) {
			return;
		}

		revoking = true;
		try {
			const response = await authApi.revokeTokenById(revokeTokenId, { reason: revokeReason });
			notifications.success(response.message);
			revokeTokenId = '';
			revokeReason = '';
		} catch (error: any) {
			notifications.error(`Failed to revoke token: ${error.message}`);
		} finally {
			revoking = false;
		}
	}

	async function handleCleanup() {
		cleaning = true;
		try {
			const response = await authApi.cleanupExpiredTokens();
			notifications.success(`${response.message}`);
		} catch (error: any) {
			notifications.error(`Failed to cleanup tokens: ${error.message}`);
		} finally {
			cleaning = false;
		}
	}
</script>

<svelte:head>
	<title>Token Administration - Pangolin</title>
</svelte:head>

<div class="space-y-6">
	<div>
		<h1 class="text-3xl font-bold text-gray-900 dark:text-white">Token Administration</h1>
		<p class="mt-2 text-gray-600 dark:text-gray-400">
			Generate tokens for service accounts or testing, and manage revocations.
		</p>
	</div>

	<div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
		<!-- Generate Token Card -->
		<Card>
			<h2 class="text-xl font-semibold mb-4 text-gray-900 dark:text-white">Generate Token</h2>
			<div class="space-y-4">
				<div>
					<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Tenant <span class="text-red-500">*</span></label>
					{#if loadingTenants}
						<div class="animate-pulse h-10 bg-gray-200 dark:bg-gray-700 rounded"></div>
					{:else}
						<select 
							bind:value={generateForm.tenant_id}
							class="w-full px-3 py-2 border rounded-md dark:bg-gray-800 dark:border-gray-700 dark:text-white focus:ring-primary-500 focus:border-primary-500"
						>
							<option value="" disabled>Select a tenant</option>
							{#each tenants as tenant}
								<option value={tenant.id}>{tenant.name} ({tenant.id})</option>
							{/each}
						</select>
					{/if}
				</div>

				<Input 
					label="Username (Optional)" 
					bind:value={generateForm.username} 
					placeholder="e.g. service-worker"
				/>

				<Input 
					label="Roles (Optional, comma separated)" 
					bind:value={rolesInput} 
					placeholder="e.g. TenantAdmin, Writer"
				/>

				<div>
					<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Expires In</label>
					<select 
						bind:value={selectedExpiry}
						class="w-full px-3 py-2 border rounded-md dark:bg-gray-800 dark:border-gray-700 dark:text-white focus:ring-primary-500 focus:border-primary-500"
					>
						{#each expiryOptions as option}
							<option value={option.value}>{option.label}</option>
						{/each}
					</select>
				</div>

				<div class="pt-2">
					<Button 
						variant="primary" 
						fullWidth={true} 
						disabled={generating || !generateForm.tenant_id}
						on:click={handleGenerate}
					>
						{generating ? 'Generating...' : 'Generate Token'}
					</Button>
				</div>
			</div>
		</Card>

		<div class="space-y-6">
			<!-- Revoke Token Card -->
			<Card>
				<h2 class="text-xl font-semibold mb-4 text-gray-900 dark:text-white">Revoke Token</h2>
				<div class="space-y-4">
					<Input 
						label="Token ID (UUID) *" 
						bind:value={revokeTokenId} 
						placeholder="00000000-0000-0000-0000-000000000000"
					/>
					
					<Input 
						label="Reason" 
						bind:value={revokeReason} 
						placeholder="e.g. Key compromise"
					/>

					<div class="pt-2">
						<Button 
							variant="error" 
							fullWidth={true} 
							disabled={revoking || !revokeTokenId}
							on:click={handleRevoke}
						>
							{revoking ? 'Revoking...' : 'Revoke Token'}
						</Button>
					</div>
				</div>
			</Card>

			<!-- Maintenance Card -->
			<Card>
				<h2 class="text-xl font-semibold mb-4 text-gray-900 dark:text-white">Maintenance</h2>
				<p class="text-sm text-gray-600 dark:text-gray-400 mb-4">
					Cleanup expired tokens from the revocation list (blacklist) to free up storage.
				</p>
				<Button 
					variant="secondary" 
					fullWidth={true} 
					disabled={cleaning}
					on:click={handleCleanup}
				>
					{cleaning ? 'Cleaning...' : 'Cleanup Expired Tokens'}
				</Button>
			</Card>
		</div>
	</div>
</div>

<Modal open={showTokenModal} title="Token Generated" on:close={closeTokenModal}>
	<div class="space-y-4">
		<div class="bg-yellow-50 dark:bg-yellow-900/30 p-4 rounded-md border border-yellow-200 dark:border-yellow-800">
			<p class="text-sm text-yellow-800 dark:text-yellow-200 font-medium">
				Warning: This token will only be shown once. Please copy it and store it securely.
			</p>
		</div>

		<div>
			<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Access Token</label>
			<div class="relative">
				<textarea 
					class="w-full px-3 py-2 border rounded-md dark:bg-gray-800 dark:border-gray-700 dark:text-white font-mono text-sm h-32 pr-12 resize-none"
					readonly
					value={generatedToken}
				></textarea>
				<button 
					class="absolute top-2 right-2 p-2 text-gray-500 hover:text-gray-700 dark:hover:text-gray-300 bg-white dark:bg-gray-800 rounded border border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
					on:click={copyToken}
					title="Copy to clipboard"
				>
					<span class="material-icons text-sm">content_copy</span>
				</button>
			</div>
		</div>

		<div class="text-sm text-gray-500 dark:text-gray-400">
			Expires in: {expiryOptions.find(opt => opt.value === selectedExpiry)?.label}
		</div>
	</div>
	<div slot="footer">
		<Button on:click={closeTokenModal}>Close</Button>
	</div>
</Modal>
