<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import Card from '$lib/components/ui/Card.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import { notifications } from '$lib/stores/notifications';
	import { 
		mergesApi, 
		type MergeOperation, 
		type MergeConflict 
	} from '$lib/api/merges';

	let operation: MergeOperation | null = null;
	let conflicts: MergeConflict[] = [];
	let loading = true;
	let processing = false;

    $: catalogName = $page.params.name ?? '';
    $: operationId = $page.params.id ?? '';

	onMount(async () => {
        if (operationId) {
		    await loadData();
        }
	});

	async function loadData() {
		loading = true;
		try {
			const [op, confs] = await Promise.all([
				mergesApi.get(operationId),
				mergesApi.listConflicts(operationId)
			]);
			operation = op;
			conflicts = confs;
            
            // If completed or aborted, redirect? Or show status.
            if (operation.status === 'completed' || operation.status === 'aborted') {
                notifications.info(`Merge operation is ${operation.status}`);
            }

		} catch (error: any) {
			console.error('Failed to load merge data:', error);
			notifications.error(`Failed to load merge data: ${error.message}`);
		} finally {
			loading = false;
		}
	}

	async function resolveConflict(conflictId: string, strategy: 'source' | 'target') {
		processing = true;
		try {
			await mergesApi.resolveConflict(conflictId, { strategy });
			notifications.success('Conflict resolved');
			// Refresh conflicts list
             // Optimistic update
             conflicts = conflicts.filter(c => c.id !== conflictId);
             
             // Check if all resolved
             if (conflicts.length === 0) {
                 await loadData(); // Reload to get updated status
             }
		} catch (error: any) {
			notifications.error(`Failed to resolve conflict: ${error.message}`);
		} finally {
			processing = false;
		}
	}

	async function completeMerge() {
		processing = true;
		try {
			await mergesApi.complete(operationId);
			notifications.success('Merge completed successfully');
			goto(`/catalogs/${encodeURIComponent(catalogName)}`);
		} catch (error: any) {
			notifications.error(`Failed to complete merge: ${error.message}`);
		} finally {
			processing = false;
		}
	}

	async function abortMerge() {
        if (!confirm('Are you sure you want to abort this merge? All resolution progress will be lost.')) return;
        
		processing = true;
		try {
			await mergesApi.abort(operationId);
			notifications.success('Merge aborted');
			goto(`/catalogs/${encodeURIComponent(catalogName)}`);
		} catch (error: any) {
			notifications.error(`Failed to abort merge: ${error.message}`);
		} finally {
			processing = false;
		}
	}
</script>

<svelte:head>
	<title>Resolve Conflicts - Pangolin</title>
</svelte:head>

<div class="space-y-6 max-w-4xl mx-auto">
	<div class="flex items-center justify-between">
		<div>
			<div class="flex items-center gap-3">
				<button
					on:click={() => goto(`/catalogs/${encodeURIComponent(catalogName)}`)}
					class="text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white"
				>
					← Back to Catalog
				</button>
				<h1 class="text-3xl font-bold text-gray-900 dark:text-white">
					Resolve Conflicts
				</h1>
			</div>
            {#if operation}
			<p class="mt-2 text-gray-600 dark:text-gray-400">
				Merging <span class="font-mono font-bold">{operation.source_branch}</span> into <span class="font-mono font-bold">{operation.target_branch}</span>
			</p>
            {/if}
		</div>
		<div class="flex gap-3">
			<Button variant="ghost" on:click={abortMerge} disabled={processing}>
				Abort Merge
			</Button>
			<Button 
                variant="primary" 
                on:click={completeMerge} 
                disabled={processing || conflicts.length > 0}
            >
				Complete Merge
			</Button>
		</div>
	</div>

	{#if loading}
		<div class="flex justify-center py-12">
			<div class="w-8 h-8 border-3 border-primary-600 border-t-transparent rounded-full animate-spin"></div>
		</div>
	{:else if conflicts.length === 0 && operation?.status !== 'ready'}
        <Card>
            <div class="text-center py-8">
                <span class="material-icons text-4xl text-green-500 mb-2">check_circle</span>
                <p class="text-lg font-medium text-gray-900 dark:text-white">All conflicts resolved!</p>
                <p class="text-gray-600 dark:text-gray-400 mt-1">You can now complete the merge operation.</p>
                 <div class="mt-4">
                     	<Button 
                            variant="primary" 
                            on:click={completeMerge} 
                            disabled={processing}
                        >
                            Complete Merge
                        </Button>
                 </div>
            </div>
        </Card>
    {:else}
		<div class="space-y-4">
            <h2 class="text-xl font-semibold text-gray-900 dark:text-white">
                Outstanding Conflicts ({conflicts.length})
            </h2>
			{#each conflicts as conflict (conflict.id)}
				<Card>
					<div class="flex flex-col md:flex-row md:items-start justify-between gap-4">
						<div class="flex-1">
							<div class="flex items-center gap-2 mb-2">
								<span class="px-2 py-0.5 rounded text-xs font-bold uppercase bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200">
									{conflict.conflict_type.replace(/_/g, ' ')}
								</span>
                                {#if conflict.asset_name}
								    <span class="font-mono text-sm font-semibold">{conflict.asset_name}</span>
                                {/if}
							</div>
							<p class="text-gray-700 dark:text-gray-300 mb-4">
                                {conflict.description}
                            </p>
                            
                            <!-- Detailed Diff Visualization could go here -->
						</div>
						
						<div class="flex flex-col gap-2 min-w-[150px]">
							<span class="text-xs font-medium text-gray-500 uppercase tracking-wider mb-1">Resolution</span>
							<Button 
                                size="sm" 
                                variant="secondary"
                                on:click={() => resolveConflict(conflict.id, 'source')}
                                disabled={processing}
                            >
								Keep Source
							</Button>
							<Button 
                                size="sm" 
                                variant="secondary"
                                on:click={() => resolveConflict(conflict.id, 'target')}
                                disabled={processing}
                            >
								Keep Target
							</Button>
						</div>
					</div>
				</Card>
			{/each}
		</div>
	{/if}
</div>
