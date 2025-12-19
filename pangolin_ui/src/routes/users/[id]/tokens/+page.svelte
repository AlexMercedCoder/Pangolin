<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { authApi } from '$lib/api/auth';
	import { usersApi, type User } from '$lib/api/users';
	import { tenantsApi, type Tenant } from '$lib/api/tenants';
	import { notifications } from '$lib/stores/notifications';
	import Card from '$lib/components/ui/Card.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';

	const userId = $page.params.id as string;
	let user: User | null = null;
	let tenants: Tenant[] = [];
	
	let loading = true;
	let generating = false;
	
	let selectedTenantId = '';
	let selectedExpiry = 24;
	let generatedToken: string | null = null;
	let showTokenModal = false;

	let expiryOptions = [
		{ value: 1, label: '1 Hour' },
		{ value: 24, label: '24 Hours' },
		{ value: 168, label: '7 Days' },
		{ value: 720, label: '30 Days' },
		{ value: 8760, label: '1 Year' }
	];

	onMount(async () => {
		await loadData();
	});

	async function loadData() {
		loading = true;
		try {
			const [userData, tenantsData] = await Promise.all([
				usersApi.get(userId),
				tenantsApi.list()
			]);
			user = userData;
			tenants = tenantsData;
			
			// Pre-select tenant if user belongs to one
			if (user?.tenant_id && tenants.find(t => t.id === user?.tenant_id)) {
				selectedTenantId = user.tenant_id;
			} else if (tenants.length > 0) {
				selectedTenantId = tenants[0].id;
			}
		} catch (error: any) {
			notifications.error(`Failed to load data: ${error.message}`);
		} finally {
			loading = false;
		}
	}

	async function handleGenerate() {
		if (!selectedTenantId) {
			notifications.error('Tenant is required');
			return;
		}

		if (!user) return;

		generating = true;
		try {
			const response = await authApi.generateToken({
				tenant_id: selectedTenantId,
				username: user.username,
				// We don't specify roles, letting the backend assign the user's current role
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
</script>

<svelte:head>
	<title>User Tokens - Pangolin</title>
</svelte:head>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-3xl font-bold text-gray-900 dark:text-white">Access Tokens</h1>
			{#if user}
				<p class="mt-2 text-gray-600 dark:text-gray-400">
					Generate API tokens for <strong>{user.username}</strong>
				</p>
			{/if}
		</div>
	</div>

	{#if loading}
		<div class="flex justify-center py-12">
			<div class="w-8 h-8 border-3 border-primary-600 border-t-transparent rounded-full animate-spin"></div>
		</div>
	{:else if user}
		<div class="max-w-xl">
			<Card>
				<h2 class="text-xl font-semibold mb-4 text-gray-900 dark:text-white">Generate New Token</h2>
				<div class="space-y-4">
					<div>
						<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Tenant <span class="text-red-500">*</span></label>
						<select 
							bind:value={selectedTenantId}
							class="w-full px-3 py-2 border rounded-md dark:bg-gray-800 dark:border-gray-700 dark:text-white focus:ring-primary-500 focus:border-primary-500"
						>
							<option value="" disabled>Select a tenant</option>
							{#each tenants as tenant}
								<option value={tenant.id}>{tenant.name}</option>
							{/each}
						</select>
						{#if !selectedTenantId}
							<p class="mt-1 text-xs text-red-500">Please select a tenant context for this token.</p>
						{/if}
					</div>

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
							disabled={generating || !selectedTenantId}
							on:click={handleGenerate}
						>
							{generating ? 'Generating...' : 'Generate Token'}
						</Button>
					</div>
				</div>
			</Card>
		</div>
	{:else}
		<div class="text-center py-12 text-gray-500">
			User not found.
		</div>
	{/if}
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
