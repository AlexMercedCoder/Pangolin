<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import Card from '$lib/components/ui/Card.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
	import { tenantsApi, type Tenant } from '$lib/api/tenants';
	import { catalogsApi } from '$lib/api/catalogs';
	import { warehousesApi } from '$lib/api/warehouses';
	import { notifications } from '$lib/stores/notifications';

	let tenant: Tenant | null = null;
	let loading = true;
	let showDeleteDialog = false;
	let deleting = false;
	let catalogCount = 0;
	let warehouseCount = 0;

	$: tenantId = $page.params.id;

	onMount(async () => {
		await loadTenant();
	});

	async function loadTenant() {
		if (!tenantId) return;

		loading = true;
		try {
			tenant = await tenantsApi.get(tenantId);
			
			// Load related counts (this could be optimized with a single API call)
			const [catalogs, warehouses] = await Promise.all([
				catalogsApi.list(),
				warehousesApi.list()
			]);
			catalogCount = catalogs.length;
			warehouseCount = warehouses.length;
		} catch (error: any) {
			notifications.error(`Failed to load tenant: ${error.message}`);
			goto('/tenants');
		}
		loading = false;
	}

	async function handleDelete() {
		if (!tenantId) return;

		deleting = true;
		try {
			await tenantsApi.delete(tenantId);
			notifications.success(`Tenant "${tenant?.name}" deleted successfully`);
			goto('/tenants');
		} catch (error: any) {
			notifications.error(`Failed to delete tenant: ${error.message}`);
		}
		deleting = false;
	}
</script>

<svelte:head>
	<title>{tenant?.name || 'Tenant'} - Pangolin</title>
</svelte:head>

<div class="space-y-6">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<div>
			<div class="flex items-center gap-3">
				<button
					on:click={() => goto('/tenants')}
					class="text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white"
				>
					← Back
				</button>
				<h1 class="text-3xl font-bold text-gray-900 dark:text-white">
					{tenant?.name || 'Loading...'}
				</h1>
			</div>
			<p class="mt-2 text-gray-600 dark:text-gray-400">Tenant details and configuration</p>
		</div>
		<div class="flex items-center gap-3">
			<Button on:click={() => goto(`/tenants/${tenantId}/edit`)} disabled={loading}>
				Edit Tenant
			</Button>
			<Button variant="error" on:click={() => (showDeleteDialog = true)} disabled={loading}>
				Delete Tenant
			</Button>
		</div>
	</div>

	{#if loading}
		<Card>
			<div class="flex items-center justify-center py-12">
				<div class="w-8 h-8 border-3 border-primary-600 border-t-transparent rounded-full animate-spin"></div>
			</div>
		</Card>
	{:else if tenant}
		<!-- Details Card -->
		<Card>
			<h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">Details</h3>
			<dl class="grid grid-cols-1 md:grid-cols-2 gap-4">
				<div>
					<dt class="text-sm font-medium text-gray-500 dark:text-gray-400">ID</dt>
					<dd class="mt-1 text-sm text-gray-900 dark:text-white font-mono">{tenant.id}</dd>
				</div>
				<div>
					<dt class="text-sm font-medium text-gray-500 dark:text-gray-400">Name</dt>
					<dd class="mt-1 text-sm text-gray-900 dark:text-white">{tenant.name}</dd>
				</div>
				{#if tenant.description}
					<div class="md:col-span-2">
						<dt class="text-sm font-medium text-gray-500 dark:text-gray-400">Description</dt>
						<dd class="mt-1 text-sm text-gray-900 dark:text-white">{tenant.description}</dd>
					</div>
				{/if}
				<div>
					<dt class="text-sm font-medium text-gray-500 dark:text-gray-400">Created</dt>
					<dd class="mt-1 text-sm text-gray-900 dark:text-white">
						{tenant.created_at ? new Date(tenant.created_at).toLocaleString() : 'N/A'}
					</dd>
				</div>
			</dl>
		</Card>

		<!-- Statistics Card -->
		<Card>
			<h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">Statistics</h3>
			<div class="grid grid-cols-1 md:grid-cols-2 gap-6">
				<div class="text-center p-4 bg-gray-50 dark:bg-gray-800 rounded-lg">
					<div class="text-3xl font-bold text-primary-600 dark:text-primary-400">
						{catalogCount}
					</div>
					<div class="mt-1 text-sm text-gray-600 dark:text-gray-400">Catalogs</div>
				</div>
				<div class="text-center p-4 bg-gray-50 dark:bg-gray-800 rounded-lg">
					<div class="text-3xl font-bold text-primary-600 dark:text-primary-400">
						{warehouseCount}
					</div>
					<div class="mt-1 text-sm text-gray-600 dark:text-gray-400">Warehouses</div>
				</div>
			</div>
		</Card>

		<!-- Properties Card -->
		{#if tenant.properties && Object.keys(tenant.properties).length > 0}
			<Card>
				<h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">Properties</h3>
				<dl class="space-y-3">
					{#each Object.entries(tenant.properties) as [key, value]}
						<div class="flex items-start gap-4">
							<dt class="text-sm font-medium text-gray-500 dark:text-gray-400 min-w-[200px]">
								{key}
							</dt>
							<dd class="text-sm text-gray-900 dark:text-white flex-1">
								<code class="bg-gray-100 dark:bg-gray-800 px-2 py-1 rounded">{value}</code>
							</dd>
						</div>
					{/each}
				</dl>
			</Card>
		{/if}
	{/if}
</div>

<!-- Delete Confirmation Dialog -->
<ConfirmDialog
	bind:open={showDeleteDialog}
	title="Delete Tenant"
	message="Are you sure you want to delete this tenant? This action cannot be undone and will remove all associated catalogs, warehouses, and users."
	confirmText={deleting ? 'Deleting...' : 'Delete'}
	variant="danger"
	onConfirm={handleDelete}
	loading={deleting}
/>
