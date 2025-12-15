<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import Card from '$lib/components/ui/Card.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import DataTable from '$lib/components/ui/DataTable.svelte';
	import { branchesApi, type Branch } from '$lib/api/branches';
	import { catalogsApi } from '$lib/api/catalogs';
	import { notifications } from '$lib/stores/notifications';

	let branches: Branch[] = [];
	let catalogs: any[] = [];
	let loading = true;
	let selectedCatalog = '';

	const columns = [
		{ key: 'name', label: 'Branch Name', sortable: true },
		{ key: 'branch_type', label: 'Type', sortable: true },
		{ key: 'from_branch', label: 'From Branch', sortable: true },
		{ key: 'assets', label: 'Assets', sortable: false },
		{ key: 'created_at', label: 'Created', sortable: true },
	];

	onMount(async () => {
		await loadCatalogs();
	});

	async function loadCatalogs() {
		loading = true;
		try {
			catalogs = await catalogsApi.list();
			if (catalogs.length > 0) {
				selectedCatalog = catalogs[0].name;
				await loadBranches();
			}
		} catch (error: any) {
			notifications.error(`Failed to load catalogs: ${error.message}`);
		}
		loading = false;
	}

	async function loadBranches() {
		if (!selectedCatalog) return;

		loading = true;
		try {
			const allBranches = await branchesApi.list(selectedCatalog);
			console.log('All branches from API:', allBranches);
			console.log('Selected catalog:', selectedCatalog);
			
			// Filter by selected catalog
			branches = allBranches;
			console.log('Filtered branches:', branches);
			
			if (allBranches.length > 0 && branches.length === 0) {
				notifications.info(`No branches found for catalog "${selectedCatalog}". Total branches: ${allBranches.length}`);
			}
		} catch (error: any) {
			console.error('Error loading branches:', error);
			notifications.error(`Failed to load branches: ${error.message}`);
			branches = [];
		}
		loading = false;
	}

	function handleRowClick(event: CustomEvent) {
		const branch = event.detail;
		goto(`/branches/${encodeURIComponent(branch.catalog)}/${encodeURIComponent(branch.name)}`);
	}

	function handleCatalogChange() {
		loadBranches();
	}
</script>

<svelte:head>
	<title>Branches - Pangolin</title>
</svelte:head>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-3xl font-bold text-gray-900 dark:text-white">Branch Management</h1>
			<p class="mt-2 text-gray-600 dark:text-gray-400">
				Manage catalog branches for isolated development and experimentation
			</p>
		</div>
		<Button on:click={() => goto('/branches/new')}>
			<span class="text-lg mr-2">+</span>
			Create Branch
		</Button>
	</div>

	{#if catalogs.length > 0}
		<Card>
			<div class="mb-4">
				<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
					Filter by Catalog
				</label>
				<select
					bind:value={selectedCatalog}
					on:change={handleCatalogChange}
					class="block w-full md:w-64 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg shadow-sm focus:outline-none focus:ring-2 focus:ring-primary-500 bg-white dark:bg-gray-800 dark:text-white"
				>
					{#each catalogs as catalog}
						<option value={catalog.name}>{catalog.name}</option>
					{/each}
				</select>
			</div>
		</Card>
	{/if}

	<Card>
		<DataTable
			{columns}
			data={branches}
			{loading}
			emptyMessage="No branches found. Create your first branch to enable isolated development."
			searchPlaceholder="Search branches..."
			on:rowClick={handleRowClick}
		>
			<svelte:fragment slot="cell" let:row let:column>
				{#if column.key === 'branch_type'}
					<span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {row.branch_type === 'production' ? 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200' : 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200'}">
						{row.branch_type}
					</span>
				{:else if column.key === 'assets'}
					<span class="text-sm text-gray-600 dark:text-gray-400">
						{row.assets?.length || 0} assets
					</span>
				{:else if column.key === 'created_at'}
					<span class="text-sm text-gray-600 dark:text-gray-400">
						{new Date(row.created_at).toLocaleDateString()}
					</span>
				{:else if column.key === 'from_branch'}
					<span class="text-sm text-gray-600 dark:text-gray-400">
						{row.from_branch || 'main'}
					</span>
				{:else}
					{row[column.key] ?? '-'}
				{/if}
			</svelte:fragment>
		</DataTable>
	</Card>
</div>
