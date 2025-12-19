<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import { goto } from '$app/navigation';
	import Modal from '$lib/components/ui/Modal.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import { branchesApi, type Branch } from '$lib/api/branches';
	import { notifications } from '$lib/stores/notifications';

	export let open = false;
	export let catalog: string;
	export let branches: Branch[] = [];

	const dispatch = createEventDispatcher();

	let sourceBranch = '';
	let targetBranch = 'main';
	let loading = false;
	let error = '';

	$: availableSources = branches.filter(b => b.name !== targetBranch);
	$: availableTargets = branches.filter(b => b.name !== sourceBranch);

	async function handleMerge() {
		if (!sourceBranch || !targetBranch) {
			error = 'Please select both source and target branches';
			return;
		}

		loading = true;
		error = '';

		try {
			const result = await branchesApi.merge({
				catalog,
				source_branch: sourceBranch,
				target_branch: targetBranch
			});

			if (result.status === 'conflicted') {
				notifications.warning(`Merge conflict detected. Redirecting to resolution...`);
				dispatch('close');
				goto(`/catalogs/${encodeURIComponent(catalog)}/merges/${result.operation_id}`);
			} else {
				notifications.success('Branches merged successfully');
				dispatch('success');
				dispatch('close');
			}
		} catch (err: any) {
			console.error('Merge failed:', err);
			error = err.message || 'Failed to merge branches';
		} finally {
			loading = false;
		}
	}

	function handleClose() {
		open = false;
		dispatch('close');
		// Reset form
		sourceBranch = '';
		targetBranch = 'main';
		error = '';
	}
</script>

<Modal {open} title="Merge Branches" on:close={handleClose}>
	<div class="space-y-4">
		{#if error}
			<div class="p-3 text-sm text-red-700 bg-red-100 rounded-lg dark:bg-red-900/30 dark:text-red-400">
				{error}
			</div>
		{/if}

		<div class="space-y-2">
			<label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
				Source Branch
			</label>
			<select
				bind:value={sourceBranch}
				class="block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg shadow-sm focus:outline-none focus:ring-2 focus:ring-primary-500 bg-white dark:bg-gray-800 dark:text-white"
			>
				<option value="" disabled>Select source branch...</option>
				{#each availableSources as branch}
					<option value={branch.name}>{branch.name}</option>
				{/each}
			</select>
			<p class="text-xs text-gray-500">The branch with changes you want to merge.</p>
		</div>

		<div class="flex justify-center">
			<span class="material-icons text-gray-400">arrow_downward</span>
		</div>

		<div class="space-y-2">
			<label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
				Target Branch
			</label>
			<select
				bind:value={targetBranch}
				class="block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg shadow-sm focus:outline-none focus:ring-2 focus:ring-primary-500 bg-white dark:bg-gray-800 dark:text-white"
			>
				<option value="" disabled>Select target branch...</option>
				{#each availableTargets as branch}
					<option value={branch.name}>{branch.name}</option>
				{/each}
			</select>
			<p class="text-xs text-gray-500">The branch that will receive the changes.</p>
		</div>
	</div>

	<div slot="footer" class="flex justify-end gap-3">
		<Button variant="ghost" on:click={handleClose}>Cancel</Button>
		<Button 
			on:click={handleMerge} 
			disabled={!sourceBranch || !targetBranch || loading}
			{loading}
		>
			Merge Branches
		</Button>
	</div>
</Modal>
