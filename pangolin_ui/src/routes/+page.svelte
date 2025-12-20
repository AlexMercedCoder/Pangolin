<script lang="ts">
	import Card from '$lib/components/ui/Card.svelte';
	import GettingStarted from '$lib/components/dashboard/GettingStarted.svelte';
    import Button from '$lib/components/ui/Button.svelte';
    import Modal from '$lib/components/ui/Modal.svelte';
    import Input from '$lib/components/ui/Input.svelte';
    import { user, token } from '$lib/stores/auth';
    import { authApi } from '$lib/api/auth';
    import { goto } from '$app/navigation';

    let showTokenModal = false;
    let newToken = '';
    let tokenExpiry = '';
    let generatingToken = false;
    let copySuccess = false;

    async function handleGenerateToken() {
        if (!$user) {
            console.error('Generatetoken: No user in store');
            alert('User session invalid. Please relogin.');
            return;
        }
        
        // Handle kebab-case vs camelCase mismatch from backend serialization
        let tenantId = $user.tenant_id || ($user as any)['tenant-id'];
        
        // If user is root (no tenant_id), we might need to handle it differently
        // For now, if they are root, they likely want a system token, or we can prompt them.
        // But the API might require a tenant_id for MOST operation tokens.
        // However, root users might need to specify WHICH tenant they want a token for,
        // OR the backend might accept a token without tenant_id for root operations?
        // Checking the error "User has no tenant_id", implies the frontend block above.
        
        // FIX: If root user, we should probably redirect them to the admin token page
        // where they can select a tenant, OR we pick the first available tenant, 
        // OR we just alert them to use the admin tools.
        
        // Check detailed user state
        console.log('GenerateToken Debug:', { tenantId, role: $user.role, user: $user });

        const role = $user.role;
        const isRoot = role === 'root' || (typeof role === 'string' && role.toLowerCase() === 'root') || 
                       (typeof role === 'object' && Object.values(role).includes('root'));

        if (!tenantId && isRoot) {
             // For root users without a tenant, redirect to admin token page
             alert('Root users must select a tenant to generate a token. You will be redirected to the Admin Token page.');
             goto('/admin/tokens');
             return;
        }

        if (!tenantId) {
            console.error('Generatetoken: User has no tenant_id', $user);
            alert('Your user is not associated with a tenant.');
            return;
        }
        
        generatingToken = true;
        try {
            // Generate a token for the current user (defaulting to 24h)
            const res = await authApi.generateToken({
                tenant_id: tenantId,
                username: $user.username,
                expires_in_hours: 24
            });
            newToken = res.token;
            tokenExpiry = new Date(res.expires_at).toLocaleString();
            showTokenModal = true;
        } catch (e) {
            console.error(e);
            alert('Failed to generate token');
        } finally {
            generatingToken = false;
        }
    }

    function copyToken() {
        navigator.clipboard.writeText(newToken);
        copySuccess = true;
        setTimeout(() => copySuccess = false, 2000);
    }
</script>

<svelte:head>
	<title>Dashboard - Pangolin</title>
</svelte:head>

<div class="space-y-6">
	<div class="flex justify-between items-center">
		<div>
			<h1 class="text-3xl font-bold text-gray-900 dark:text-white">Dashboard</h1>
			<p class="mt-2 text-gray-600 dark:text-gray-400">Welcome to Pangolin Lakehouse Catalog</p>
		</div>
        <div class="actions">
             <Button variant="outline" on:click={handleGenerateToken} disabled={generatingToken}>
                {generatingToken ? 'Generating...' : 'Generate New Token'}
            </Button>
        </div>
	</div>

	<div class="grid grid-cols-1 gap-6">
		<GettingStarted />
	</div>
</div>

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
