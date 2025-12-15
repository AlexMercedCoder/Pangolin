<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import Card from '$lib/components/ui/Card.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
	import { branchesApi, type Branch } from '$lib/api/branches';
	import { notifications } from '$lib/stores/notifications';

	let branch: Branch | null = null;
	let loading = true;
	let showDeleteDialog = false;
	let showMergeDialog = false;
	let deleting = false;
	let merging = false;
	let targetBranch = 'main';

	$: catalogName = $page.params.catalog;
	$: branchName = $page.params.name;

	onMount(async () => {
		await loadBranch();
	});

	async function loadBranch() {
		if (!catalogName || !branchName) return;

		loading = true;
		try {
			branch = await branchesApi.get(catalogName, branchName);
		} catch (error: any) {
			notifications.error(`Failed to load branch: ${error.message}`);
			goto('/branches');
		}
		loading = false;
	}

	async function handleDelete() {
		if (!catalogName || !branchName) return;

		deleting = true;
		try {
			await branchesApi.delete(catalogName, branchName);
			notifications.success(`Branch "${branchName}" deleted successfully`);
			goto('/branches');
		} catch (error: any) {
			notifications.error(`Failed to delete branch: ${error.message}`);
		}
		deleting = false;
	}

	async function handleMerge() {
		if (!catalogName || !branchName) return;

		merging = true;
		try {
			await branchesApi.merge({
				catalog: catalogName,
				source_branch: branchName,
				target_branch: targetBranch
			});
			notifications.success(`Branch "${branchName}" merged into "${targetBranch}" successfully`);
			showMergeDialog = false;
			goto('/branches');
		} catch (error: any) {
			notifications.error(`Failed to merge branch: ${error.message}`);
		}
		merging = false;
	}
</script>

<svelte:head>
	<title>{branch?.name || 'Branch'} - Pangolin</title>
</svelte:head>

<div class="space-y-6">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<div>
			<div class="flex items-center gap-3">
				<button
					on:click={() => goto('/branches')}
					class="text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white"
				>
					← Back
				</button>
				<h1 class="text-3xl font-bold text-gray-900 dark:text-white">
					{branch?.name || 'Loading...'}
				</h1>
				{#if branch}
					<span class="inline-flex items-center px-3 py-1 rounded-full text-sm font-medium {branch.branch_type === 'production' ? 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200' : 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200'}">
						{branch.branch_type}
					</span>
				{/if}
			</div>
			<p class="mt-2 text-gray-600 dark:text-gray-400">Branch details and operations</p>
		</div>
		<div class="flex items-center gap-3">
			{#if branch && branch.name !== 'main'}
				<Button on:click={() => (showMergeDialog = true)} disabled={loading}>
					Merge Branch
				</Button>
				<Button variant="error" on:click={() => (showDeleteDialog = true)} disabled={loading}>
					Delete Branch
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
	{:else if branch}
		<!-- Details Card -->
		<Card>
			<h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">Details</h3>
			<dl class="grid grid-cols-1 md:grid-cols-2 gap-4">
				<div>
					<dt class="text-sm font-medium text-gray-500 dark:text-gray-400">ID</dt>
					<dd class="mt-1 text-sm text-gray-900 dark:text-white font-mono">{branch.id}</dd>
				</div>
				<div>
					<dt class="text-sm font-medium text-gray-500 dark:text-gray-400">Name</dt>
					<dd class="mt-1 text-sm text-gray-900 dark:text-white">{branch.name}</dd>
				</div>
				<div>
					<dt class="text-sm font-medium text-gray-500 dark:text-gray-400">Catalog</dt>
					<dd class="mt-1 text-sm text-gray-900 dark:text-white">
						<a
							href="/catalogs/{encodeURIComponent(branch.catalog)}"
							class="text-primary-600 hover:text-primary-700 hover:underline"
						>
							{branch.catalog}
						</a>
					</dd>
				</div>
				<div>
					<dt class="text-sm font-medium text-gray-500 dark:text-gray-400">Type</dt>
					<dd class="mt-1 text-sm text-gray-900 dark:text-white capitalize">{branch.branch_type}</dd>
				</div>
				{#if branch.from_branch}
					<div>
						<dt class="text-sm font-medium text-gray-500 dark:text-gray-400">From Branch</dt>
						<dd class="mt-1 text-sm text-gray-900 dark:text-white">{branch.from_branch}</dd>
					</div>
				{/if}
				<div>
					<dt class="text-sm font-medium text-gray-500 dark:text-gray-400">Created</dt>
					<dd class="mt-1 text-sm text-gray-900 dark:text-white">
						{new Date(branch.created_at).toLocaleString()}
					</dd>
				</div>
			</dl>
		</Card>

		<!-- Assets Card -->
		{#if branch.assets && branch.assets.length > 0}
			<Card>
				<h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">
					Tracked Assets ({branch.assets.length})
				</h3>
				<div class="space-y-2">
					{#each branch.assets as asset}
						<div class="flex items-center gap-2 p-2 bg-gray-50 dark:bg-gray-800 rounded">
							<span class="text-sm font-mono text-gray-900 dark:text-white">{asset}</span>
						</div>
					{/each}
				</div>
			</Card>
		{:else}
			<Card>
				<h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">Tracked Assets</h3>
				<p class="text-gray-600 dark:text-gray-400">
					This branch tracks all assets in the catalog (full branching).
				</p>
			</Card>
		{/if}
	{/if}
</div>

<!-- Delete Confirmation Dialog -->
<ConfirmDialog
	bind:open={showDeleteDialog}
	title="Delete Branch"
	message="Are you sure you want to delete this branch? This action cannot be undone."
	confirmText={deleting ? 'Deleting...' : 'Delete'}
	variant="danger"
	onConfirm={handleDelete}
	loading={deleting}
/>

<!-- Merge Confirmation Dialog -->
<ConfirmDialog
	bind:open={showMergeDialog}
	title="Merge Branch"
	message="Merge '{branchName}' into '{targetBranch}'? This will apply all changes from this branch."
	confirmText={merging ? 'Merging...' : 'Merge'}
	variant="warning"
	onConfirm={handleMerge}
	loading={merging}
/>
