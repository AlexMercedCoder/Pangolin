<script lang="ts">
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import Card from '$lib/components/ui/Card.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import { branchesApi, type CreateBranchRequest } from '$lib/api/branches';
	import { catalogsApi } from '$lib/api/catalogs';
	import { notifications } from '$lib/stores/notifications';

	let loading = false;
	let catalogs: any[] = [];
	let catalogsLoading = true;

	// Form data
	let name = '';
	let catalog = '';
	let fromBranch = 'main';
	let branchType: 'experimental' | 'production' = 'experimental';
	let assets: string = '';
	let errors: Record<string, string> = {};

	onMount(async () => {
		await loadCatalogs();
	});

	async function loadCatalogs() {
		catalogsLoading = true;
		try {
			catalogs = await catalogsApi.list();
			if (catalogs.length > 0) {
				catalog = catalogs[0].name;
			}
		} catch (error: any) {
			notifications.error(`Failed to load catalogs: ${error.message}`);
		}
		catalogsLoading = false;
	}

	function validateForm(): boolean {
		errors = {};

		if (!name || name.trim().length === 0) {
			errors.name = 'Branch name is required';
		}

		if (!catalog) {
			errors.catalog = 'Catalog is required';
		}

		return Object.keys(errors).length === 0;
	}

	async function handleSubmit() {
		if (!validateForm()) return;

		loading = true;
		try {
			const request: CreateBranchRequest = {
				name: name.trim(),
				catalog,
				branch_type: branchType,
			};

			if (fromBranch && fromBranch !== 'main') {
				request.from_branch = fromBranch;
			}

			// Parse assets if provided (comma-separated)
			if (assets && assets.trim()) {
				request.assets = assets.split(',').map(a => a.trim()).filter(a => a.length > 0);
			}

			await branchesApi.create(request);
			notifications.success(`Branch "${name}" created successfully`);
			goto('/branches');
		} catch (error: any) {
			notifications.error(`Failed to create branch: ${error.message}`);
		}
		loading = false;
	}
</script>

<svelte:head>
	<title>Create Branch - Pangolin</title>
</svelte:head>

<div class="max-w-3xl mx-auto space-y-6">
	<div>
		<div class="flex items-center gap-3">
			<button
				on:click={() => goto('/branches')}
				class="text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white"
			>
				← Back
			</button>
			<h1 class="text-3xl font-bold text-gray-900 dark:text-white">Create Branch</h1>
		</div>
		<p class="mt-2 text-gray-600 dark:text-gray-400">
			Create a new branch for isolated development and experimentation
		</p>
	</div>

	<Card>
		<form on:submit|preventDefault={handleSubmit} class="space-y-6">
			<div class="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-4">
				<h3 class="text-sm font-medium text-blue-800 dark:text-blue-200 mb-2">
					About Branches
				</h3>
				<p class="text-sm text-blue-700 dark:text-blue-300">
					Branches allow you to work on catalog changes in isolation. Use <strong>experimental</strong> branches
					for testing and development, and <strong>production</strong> branches for stable releases.
				</p>
			</div>

			<Input
				label="Branch Name"
				bind:value={name}
				error={errors.name}
				required
				placeholder="feature-branch"
				helpText="A unique name for this branch"
			/>

			<div class="space-y-2">
				<label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
					Catalog
					<span class="text-error-500">*</span>
				</label>
				<select
					bind:value={catalog}
					disabled={catalogsLoading || loading}
					class="block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg shadow-sm focus:outline-none focus:ring-2 focus:ring-primary-500 bg-white dark:bg-gray-800 dark:text-white {errors.catalog ? 'border-error-500' : ''}"
				>
					{#if catalogsLoading}
						<option>Loading catalogs...</option>
					{:else if catalogs.length === 0}
						<option>No catalogs available</option>
					{:else}
						{#each catalogs as cat}
							<option value={cat.name}>{cat.name}</option>
						{/each}
					{/if}
				</select>
				{#if errors.catalog}
					<p class="text-sm text-error-600 dark:text-error-400">{errors.catalog}</p>
				{/if}
				<p class="text-sm text-gray-500 dark:text-gray-400">
					The catalog this branch belongs to
				</p>
			</div>

			<Input
				label="From Branch"
				bind:value={fromBranch}
				placeholder="main"
				helpText="The branch to create this branch from (defaults to 'main')"
			/>

			<div class="space-y-2">
				<label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
					Branch Type
				</label>
				<select
					bind:value={branchType}
					disabled={loading}
					class="block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg shadow-sm focus:outline-none focus:ring-2 focus:ring-primary-500 bg-white dark:bg-gray-800 dark:text-white"
				>
					<option value="experimental">Experimental</option>
					<option value="ingest">Ingest</option>
				</select>
				<p class="text-sm text-gray-500 dark:text-gray-400">
					<strong>Experimental:</strong> For isolated testing and experimentation. These branches are <em>not intended to be merged</em> back. Use for temporary work, prototyping, and data exploration that will be discarded.
					<br />
					<strong>Ingest:</strong> For data ingestion and integration workflows. These branches are <em>intended to be merged</em> into main or other branches. Use for ETL pipelines, data updates, and changes meant to be promoted.
					<br />
					<em>Note: Currently both types can be merged, but this metadata helps administrators understand each branch's purpose.</em>
				</p>
			</div>

			<div class="space-y-2">
				<label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
					Assets (Optional)
				</label>
				<textarea
					bind:value={assets}
					rows="3"
					placeholder="namespace.table1, namespace.table2"
					class="block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg shadow-sm focus:outline-none focus:ring-2 focus:ring-primary-500 bg-white dark:bg-gray-800 dark:text-white"
					disabled={loading}
				></textarea>
				<p class="text-sm text-gray-500 dark:text-gray-400">
					Comma-separated list of assets to track on this branch (partial branching)
				</p>
			</div>

			<div class="flex items-center gap-3 pt-4 border-t border-gray-200 dark:border-gray-700">
				<Button
					type="button"
					variant="ghost"
					on:click={() => goto('/branches')}
					disabled={loading}
				>
					Cancel
				</Button>
				<Button type="submit" loading={loading}>
					{loading ? 'Creating...' : 'Create Branch'}
				</Button>
			</div>
		</form>
	</Card>
</div>
