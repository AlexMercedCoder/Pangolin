<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import Card from '$lib/components/ui/Card.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
	import { catalogsApi, type Catalog } from '$lib/api/catalogs';
	import { notifications } from '$lib/stores/notifications';

	let catalog: Catalog | null = null;
	let loading = true;
	let showDeleteDialog = false;
	let deleting = false;

	$: catalogName = $page.params.name;

	onMount(async () => {
		await loadCatalog();
	});

	async function loadCatalog() {
		if (!catalogName) return;

		loading = true;
		const response = await catalogsApi.get(catalogName);

		if (response.error) {
			notifications.error(`Failed to load catalog: ${response.error.message}`);
			goto('/catalogs');
		} else {
			catalog = response.data || null;
		}

		loading = false;
	}

	async function handleDelete() {
		if (!catalogName) return;

		deleting = true;
		const response = await catalogsApi.delete(catalogName);

		if (response.error) {
			notifications.error(`Failed to delete catalog: ${response.error.message}`);
		} else {
			notifications.success(`Catalog "${catalogName}" deleted successfully`);
			goto('/catalogs');
		}

		deleting = false;
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
			<Button variant="error" on:click={() => (showDeleteDialog = true)} disabled={loading}>
				Delete Catalog
			</Button>
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
