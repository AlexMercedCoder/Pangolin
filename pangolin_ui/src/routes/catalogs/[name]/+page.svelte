<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import Card from '$lib/components/ui/Card.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
	import { catalogsApi, type Catalog } from '$lib/api/catalogs';
	import { branchesApi, type Branch } from '$lib/api/branches';
	import { notifications } from '$lib/stores/notifications';
    import { isTenantAdmin } from '$lib/stores/auth';

	let catalog: Catalog | null = null;
	let branches: Branch[] = [];
	let selectedBranch = 'main';
	let loading = true;
	let showDeleteDialog = false;
	let deleting = false;

	$: catalogName = $page.params.name || '';

	onMount(async () => {
		await loadCatalog();
	});

	async function loadCatalog() {
		if (!catalogName) return;

		loading = true;
		try {
			catalog = await catalogsApi.get(catalogName);
			// Load branches for this catalog
			await loadBranches();
		} catch (error: any) {
			console.error('Error loading catalog:', error);
			notifications.error(`Failed to load catalog: ${error.message}`);
			goto('/catalogs');
		} finally {
			loading = false;
		}
	}

	async function loadBranches() {
		try {
			const allBranches = await branchesApi.list();
			branches = allBranches.filter(b => b.catalog === catalogName);
		} catch (error: any) {
			console.error('Failed to load branches:', error);
			branches = [];
		}
	}

	async function handleDelete() {
		if (!catalogName) return;

		deleting = true;
		try {
			await catalogsApi.delete(catalogName);
			notifications.success(`Catalog "${catalogName}" deleted successfully`);
			// Navigate back to catalog list
			await goto('/catalogs');
		} catch (error: any) {
			console.error('Error deleting catalog:', error);
			notifications.error(`Failed to delete catalog: ${error.message}`);
		} finally {
			deleting = false;
			showDeleteDialog = false;
		}
	}
</script>

<svelte:head>
	<title>{catalog?.name || 'Catalog'} - Pangolin</title>
</svelte:head>

<div class="space-y-6">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<div>
			<div class="flex items-center gap-3">
				<button
					on:click={() => goto('/catalogs')}
					class="text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white"
				>
					← Back
				</button>
				<h1 class="text-3xl font-bold text-gray-900 dark:text-white">
					{catalog?.name || 'Loading...'}
				</h1>
			</div>
			<p class="mt-2 text-gray-600 dark:text-gray-400">Catalog details and configuration</p>
		</div>
		<div class="flex items-center gap-3">
            {#if $isTenantAdmin}
			<Button on:click={() => goto(`/catalogs/${encodeURIComponent(catalogName)}/edit`)} disabled={loading}>
				Edit Catalog
			</Button>
			<Button variant="error" on:click={() => (showDeleteDialog = true)} disabled={loading}>
				Delete Catalog
			</Button>
            {/if}
		</div>
	</div>

	{#if loading}
		<Card>
			<div class="flex items-center justify-center py-12">
				<div class="w-8 h-8 border-3 border-primary-600 border-t-transparent rounded-full animate-spin" />
			</div>
		</Card>
	{:else if catalog}
		<!-- Details Card -->
		<Card>
			<h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">Details</h3>
			<dl class="grid grid-cols-1 md:grid-cols-2 gap-4">
				<div>
					<dt class="text-sm font-medium text-gray-500 dark:text-gray-400">ID</dt>
					<dd class="mt-1 text-sm text-gray-900 dark:text-white font-mono">{catalog.id}</dd>
				</div>
				<div>
					<dt class="text-sm font-medium text-gray-500 dark:text-gray-400">Name</dt>
					<dd class="mt-1 text-sm text-gray-900 dark:text-white">{catalog.name}</dd>
				</div>
				<div>
					<dt class="text-sm font-medium text-gray-500 dark:text-gray-400">Warehouse</dt>
					<dd class="mt-1 text-sm text-gray-900 dark:text-white">
						<a
							href="/warehouses/{encodeURIComponent(catalog.warehouse_name)}"
							class="text-primary-600 hover:text-primary-700 hover:underline"
						>
							{catalog.warehouse_name}
						</a>
					</dd>
				</div>
				<div class="md:col-span-2">
					<dt class="text-sm font-medium text-gray-500 dark:text-gray-400">Storage Location</dt>
					<dd class="mt-1 text-sm text-gray-900 dark:text-white">
						<code class="bg-gray-100 dark:bg-gray-800 px-2 py-1 rounded">
							{catalog.storage_location}
						</code>
					</dd>
				</div>
			</dl>
		</Card>

		<!-- Properties Card -->
		{#if catalog.properties && Object.keys(catalog.properties).length > 0}
			<Card>
				<h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">Properties</h3>
				<dl class="space-y-3">
					{#each Object.entries(catalog.properties) as [key, value]}
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

		<!-- Namespaces Card (Placeholder for Phase 3) -->
		<Card>
			<h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">Namespaces</h3>
			<p class="text-gray-600 dark:text-gray-400">
				Namespace browsing will be available in Phase 3.
			</p>
		</Card>

		<!-- Branch Management Card -->
		<Card>
			<div class="flex items-center justify-between mb-4">
				<h3 class="text-lg font-semibold text-gray-900 dark:text-white">Branches</h3>
				<Button size="sm" on:click={() => goto('/branches/new')}>
					Create Branch
				</Button>
			</div>

			{#if branches.length === 0}
				<p class="text-gray-600 dark:text-gray-400 text-center py-8">
					No branches found for this catalog. Create a branch to enable isolated development.
				</p>
			{:else}
				<div class="space-y-3">
					<div class="space-y-2">
						<label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
							Active Branch
						</label>
						<select
							bind:value={selectedBranch}
							class="block w-full md:w-64 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg shadow-sm focus:outline-none focus:ring-2 focus:ring-primary-500 bg-white dark:bg-gray-800 dark:text-white"
						>
							{#each branches as branch}
								<option value={branch.name}>
									{branch.name} ({branch.branch_type})
								</option>
							{/each}
						</select>
						<p class="text-sm text-gray-500 dark:text-gray-400">
							Select a branch to view its details
						</p>
					</div>

					<div class="grid grid-cols-1 md:grid-cols-2 gap-3 mt-4">
						{#each branches as branch}
							<button
								on:click={() => goto(`/branches/${encodeURIComponent(branch.catalog)}/${encodeURIComponent(branch.name)}`)}
								class="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-900 rounded-lg border border-gray-200 dark:border-gray-700 hover:border-primary-500 dark:hover:border-primary-500 transition-colors text-left"
							>
								<div class="flex-1">
									<div class="font-medium text-gray-900 dark:text-white">
										{branch.name}
									</div>
									<div class="text-sm text-gray-600 dark:text-gray-400 mt-1">
										{branch.assets?.length || 0} assets • {branch.branch_type}
									</div>
								</div>
								<svg class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
								</svg>
							</button>
						{/each}
					</div>
				</div>
			{/if}
		</Card>
	{/if}
</div>

<!-- Delete Confirmation Dialog -->
<ConfirmDialog
	bind:open={showDeleteDialog}
	title="Delete Catalog"
	message="Are you sure you want to delete this catalog? This action cannot be undone and will remove all associated metadata."
	confirmText={deleting ? 'Deleting...' : 'Delete'}
	variant="danger"
	onConfirm={handleDelete}
/>
