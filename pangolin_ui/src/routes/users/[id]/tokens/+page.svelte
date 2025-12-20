<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { authApi } from '$lib/api/auth';
	import { usersApi, type User } from '$lib/api/users';
	import { tokensApi, type TokenInfo } from '$lib/api/tokens'; // Added
	import { tenantsApi, type Tenant } from '$lib/api/tenants';
	import { notifications } from '$lib/stores/notifications';
	import { authStore } from '$lib/stores/auth'; // Added
	import { get } from 'svelte/store'; // Added
	import Card from '$lib/components/ui/Card.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import TokenList from '$lib/components/tokens/TokenList.svelte'; // Added
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte'; // Added

	const userId = $page.params.id as string;
	let user: User | null = null;
	let tenants: Tenant[] = [];
	let tokens: TokenInfo[] = []; // Added

	let loading = true;
	let generating = false;
	
	let selectedTenantId = '';
	let selectedExpiry = 24;
	let generatedToken: string | null = null;
	let showTokenModal = false;
	
	let showConfirmDialog = false; // Added
	let tokenToRevoke: TokenInfo | null = null; // Added

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
			const [userData, tenantsData, tokensData] = await Promise.all([
				usersApi.get(userId),
				tenantsApi.list(),
				tokensApi.listUserTokens(userId) // Added
			]);
			user = userData;
			tenants = tenantsData;
			tokens = tokensData; // Added
			
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
    
    // Added function to reload only tokens
    async function loadTokens() {
        try {
            tokens = await tokensApi.listUserTokens(userId);
        } catch (e) {
            console.error('Failed to reload tokens', e);
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
				expires_in_hours: selectedExpiry
			});

			generatedToken = response.token;
			showTokenModal = true;
			notifications.success('Token generated successfully');
            await loadTokens(); // Reload list
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

    // Added revocation logic
    function handleRevoke(event: CustomEvent<TokenInfo>) {
        tokenToRevoke = event.detail;
        showConfirmDialog = true;
    }

    async function confirmRevoke() {
        if (!tokenToRevoke) return;
        try {
            await tokensApi.deleteToken(tokenToRevoke.id);
            notifications.success('Token revoked successfully');
            await loadTokens();
        } catch (e: any) {
             notifications.error(`Failed to revoke token: ${e.message}`);
        } finally {
            showConfirmDialog = false;
            tokenToRevoke = null;
        }
    }

    let showRotateConfirmDialog = false;
    async function confirmRotate() {
        loading = true;
        showRotateConfirmDialog = false;
        try {
            const res = await tokensApi.rotate();
            
            authStore.updateSession(res.token, {
                id: get(authStore).user?.id || '',
                username: get(authStore).user?.username || '',
                email: get(authStore).user?.email || '',
                role: get(authStore).user?.role || 'tenant-user',
                tenant_id: res.tenant_id,
                created_at: get(authStore).user?.created_at || new Date().toISOString()
            } as any);
            
            notifications.success('Token rotated successfully. Session updated.');
            await loadTokens();
        } catch (e: any) {
            notifications.error(`Failed to rotate token: ${e.message}`);
        } finally {
            loading = false;
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
		<div class="flex gap-2">
			{#if user && user.id === get(authStore).user?.id}
				<Button variant="secondary" on:click={() => showRotateConfirmDialog = true} disabled={loading}>
					Rotate Current Session
				</Button>
			{/if}
		</div>
	</div>

	{#if loading}
		<div class="flex justify-center py-12">
			<div class="w-8 h-8 border-3 border-primary-600 border-t-transparent rounded-full animate-spin"></div>
		</div>
	{:else if user}
		<div class="space-y-6">
			<Card>
				<h2 class="text-xl font-semibold mb-4 text-gray-900 dark:text-white">Active Tokens</h2>
				<TokenList 
					{tokens} 
					{loading} 
					on:revoke={handleRevoke} 
				/>
			</Card>

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
		</div>
	{:else}
		<div class="text-center py-12 text-gray-500">
			User not found.
		</div>
	{/if}
</div>

{#if showConfirmDialog}
	<ConfirmDialog
		bind:open={showConfirmDialog}
		title="Revoke Token"
		message="Are you sure you want to revoke this token? This action cannot be undone."
		confirmText="Revoke"
		variant="danger"
		onConfirm={confirmRevoke}
	/>
{/if}

{#if showRotateConfirmDialog}
	<ConfirmDialog
		bind:open={showRotateConfirmDialog}
		title="Rotate Current Session"
		message="Are you sure you want to rotate your current session token? A new token will be generated and your session will be updated locally."
		confirmText="Rotate"
		variant="primary"
		onConfirm={confirmRotate}
	/>
{/if}

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
