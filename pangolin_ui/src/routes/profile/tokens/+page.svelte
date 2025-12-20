<script lang="ts">
	import { onMount } from 'svelte';
	import { tokensApi, type TokenInfo } from '$lib/api/tokens';
	import { authApi } from '$lib/api/auth';
	import TokenList from '$lib/components/tokens/TokenList.svelte';
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
	import Notification from '$lib/components/ui/Notification.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import { authStore } from '$lib/stores/auth';
	import { get } from 'svelte/store';
	import { goto } from '$app/navigation';

	let tokens: TokenInfo[] = [];
	let loading = true;
	let error = '';
	let success = '';
	let showConfirmDialog = false;
	let tokenToRevoke: TokenInfo | null = null;
	let showRotateConfirmDialog = false;
	
	// Token generation state
	let generatingToken = false;
	let showTokenModal = false;
	let newToken = '';
	let tokenExpiry = '';
	let copySuccess = false;

	onMount(async () => {
		await loadTokens();
	});

	async function loadTokens() {
		loading = true;
		error = '';
		try {
			const auth = get(authStore);
			if (!auth.user?.id) {
				error = 'User not authenticated';
				return;
			}
			tokens = await tokensApi.listUserTokens(auth.user.id);
		} catch (e: any) {
			error = e.message || 'Failed to load tokens';
		} finally {
			loading = false;
		}
	}

	async function handleGenerateToken() {
		const auth = get(authStore);
		const user = auth.user;
		if (!user) {
			error = 'User session invalid. Please relogin.';
			return;
		}

		// Handle kebab-case vs camelCase mismatch
		let tenantId = user.tenant_id || (user as any)['tenant-id'];
		
		// Root user logic - copied from dashboard
		const role = user.role;
		const isRoot = role === 'root' || (typeof role === 'string' && role.toLowerCase() === 'root') || 
					   (typeof role === 'object' && Object.values(role).includes('root'));

		if (!tenantId && isRoot) {
			alert('Root users must select a tenant to generate a token. You will be redirected to the Admin Token page.');
			goto('/admin/tokens');
			return;
		}

		if (!tenantId) {
			error = 'Your user is not associated with a tenant.';
			return;
		}
		
		generatingToken = true;
		try {
			const res = await authApi.generateToken({
				tenant_id: tenantId,
				username: user.username,
				expires_in_hours: 24
			});
			newToken = res.token;
			tokenExpiry = new Date(res.expires_at).toLocaleString();
			showTokenModal = true;
			await loadTokens(); // Reload list
		} catch (e: any) {
			console.error(e);
			error = 'Failed to generate token';
		} finally {
			generatingToken = false;
		}
	}

	function copyToken() {
		navigator.clipboard.writeText(newToken);
		copySuccess = true;
		setTimeout(() => copySuccess = false, 2000);
	}

	function handleRevoke(event: CustomEvent<TokenInfo>) {
		tokenToRevoke = event.detail;
		showConfirmDialog = true;
	}

	async function confirmRevoke() {
		if (!tokenToRevoke) return;
		
		try {
			await tokensApi.deleteToken(tokenToRevoke.id);
			success = 'Token revoked successfully';
			showConfirmDialog = false;
			tokenToRevoke = null;
			await loadTokens();
		} catch (e: any) {
			error = e.message || 'Failed to revoke token';
		}
	}

	function cancelRevoke() {
		showConfirmDialog = false;
		tokenToRevoke = null;
	}

	function handleRotateClick() {
		showRotateConfirmDialog = true;
	}

	async function confirmRotate() {
		loading = true; // Show full page loading or just disable button
		showRotateConfirmDialog = false;
		try {
			const res = await tokensApi.rotate();
			// Update local storage / cookie if client handles it directly, 
			// but usually authStore or client.ts handles the bearer token.
			// Ideally we should update the authStore with the new token.
			authStore.updateSession(res.token, {
				id: get(authStore).user?.id || '',
				username: get(authStore).user?.username || '',
				email: get(authStore).user?.email || '',
				role: get(authStore).user?.role || 'tenant-user',
				tenant_id: res.tenant_id,
				created_at: get(authStore).user?.created_at || new Date().toISOString()
			} as any); // Cast as any because UserInfo match might be strict
			
			success = 'Token rotated successfully. Session updated.';
			await loadTokens();
		} catch (e: any) {
			error = e.message || 'Failed to rotate token';
		} finally {
			loading = false;
		}
	}
</script>

<svelte:head>
	<title>My Tokens - Pangolin</title>
</svelte:head>

<div class="container mx-auto px-4 py-8">
	<div class="flex justify-between items-start mb-8">
		<div>
			<h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100">My Tokens</h1>
			<p class="mt-2 text-gray-600 dark:text-gray-400">
				Manage your active access tokens. Revoking a token will immediately invalidate it.
			</p>
		</div>
		<div class="flex gap-2">
			<Button variant="secondary" on:click={handleRotateClick} disabled={generatingToken || loading}>
				Rotate Current Session
			</Button>
			<Button variant="primary" on:click={handleGenerateToken} disabled={generatingToken}>
				{generatingToken ? 'Generating...' : 'Generate New Token'}
			</Button>
		</div>
	</div>

	{#if error}
		<Notification type="error" message={error} on:close={() => error = ''} />
	{/if}

	{#if success}
		<Notification type="success" message={success} on:close={() => success = ''} />
	{/if}

	<div class="bg-white dark:bg-gray-800 rounded-lg shadow">
		<TokenList {tokens} {loading} on:revoke={handleRevoke} />
	</div>
</div>

{#if showConfirmDialog}
	<ConfirmDialog
		title="Revoke Token"
		message="Are you sure you want to revoke this token? This action cannot be undone and will immediately invalidate the token."
		confirmText="Revoke"
		confirmVariant="error"
		on:confirm={confirmRevoke}
		on:cancel={cancelRevoke}
	/>
{/if}

{#if showRotateConfirmDialog}
	<ConfirmDialog
		title="Rotate Current Session"
		message="Are you sure you want to rotate your current session token? A new token will be generated and your session will be updated locally."
		confirmText="Rotate"
		confirmVariant="primary"
		on:confirm={confirmRotate}
		on:cancel={() => showRotateConfirmDialog = false}
	/>
{/if}

<Modal bind:open={showTokenModal} title="New Access Token">
    <div class="space-y-4">
        <div class="p-4 bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-md">
            <p class="text-sm text-yellow-800 dark:text-yellow-200">
                <strong>Important:</strong> Copy this token now. You will not be able to see it again!
            </p>
        </div>
        
        <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Access Token</label>
            <div class="relative">
                <textarea 
                    readonly 
                    class="w-full h-32 p-3 bg-gray-50 dark:bg-gray-900 border border-gray-300 dark:border-gray-600 rounded-md font-mono text-sm break-all focus:ring-2 focus:ring-primary-500 focus:outline-none"
                    value={newToken}
                ></textarea>
                <button 
                    class="absolute top-2 right-2 p-1.5 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded hover:bg-gray-50 dark:hover:bg-gray-700 text-gray-500"
                    on:click={copyToken}
                    title="Copy to clipboard"
                >
                    {#if copySuccess}
                        <span class="material-icons text-green-500 text-sm">check</span>
                    {:else}
                        <span class="material-icons text-sm">content_copy</span>
                    {/if}
                </button>
            </div>
            <p class="mt-1 text-xs text-gray-500">Expires: {tokenExpiry}</p>
        </div>
    </div>
    
    <div slot="footer">
        <Button variant="primary" on:click={() => showTokenModal = false}>Done</Button>
    </div>
</Modal>
