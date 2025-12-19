<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import Card from '$lib/components/ui/Card.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import DataTable from '$lib/components/ui/DataTable.svelte';
	import { notifications } from '$lib/stores/notifications';
	import { mergesApi, type MergeOperation } from '$lib/api/merges';

	let operations: MergeOperation[] = [];
	let loading = true;

    $: catalogName = $page.params.name ?? '';

	const columns = [
		{ key: 'status', label: 'Status', sortable: true },
		{ key: 'source_branch', label: 'Source', sortable: true },
		{ key: 'target_branch', label: 'Target', sortable: true },
		{ key: 'conflicts_count', label: 'Conflicts', sortable: true },
		{ key: 'initiated_at', label: 'Initiated', sortable: true },
		{ key: 'actions', label: 'Actions', sortable: false },
	];

	onMount(async () => {
        if (catalogName) {
		    await loadOperations();
        }
	});

	async function loadOperations() {
		loading = true;
		try {
			operations = await mergesApi.list(catalogName);
		} catch (error: any) {
			notifications.error(`Failed to load merge operations: ${error.message}`);
		} finally {
			loading = false;
		}
	}
    
    function getStatusVariant(status: string) {
        switch (status.toLowerCase()) {
            case 'completed': return 'success';
            case 'conflicted': return 'warning';
            case 'failed': 
            case 'aborted': return 'error';
            default: return 'info';
        }
    }
    
    function getStatusColor(status: string) {
        switch (status.toLowerCase()) {
            case 'completed': return 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200';
            case 'conflicted': return 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200';
            case 'failed':
            case 'aborted': return 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200';
            default: return 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200';
        }
    }
</script>

<svelte:head>
	<title>Merge Operations - {catalogName} - Pangolin</title>
</svelte:head>

<div class="space-y-6">
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
					Merge Operations
				</h1>
			</div>
			<p class="mt-2 text-gray-600 dark:text-gray-400">
				History of merge requests and operations
			</p>
		</div>
	</div>

	<Card>
		<DataTable
			{columns}
			data={operations}
			{loading}
			emptyMessage="No merge operations found."
			searchPlaceholder="Search operations..."
		>
			<svelte:fragment slot="cell" let:row let:column>
				{#if column.key === 'status'}
					<span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {getStatusColor(row.status)}">
						{row.status}
					</span>
				{:else if column.key === 'initiated_at'}
					<span class="text-sm text-gray-600 dark:text-gray-400">
						{new Date(row.initiated_at).toLocaleDateString()} {new Date(row.initiated_at).toLocaleTimeString()}
					</span>
				{:else if column.key === 'actions'}
                    {#if row.status === 'conflicted' || row.status === 'pending' || row.status === 'ready'}
                        <Button 
                            size="sm" 
                            variant="secondary"
                            on:click={() => goto(`/catalogs/${encodeURIComponent(catalogName)}/merges/${row.id}`)}
                        >
                            {row.status === 'ready' ? 'Complete' : 'Resolve'}
                        </Button>
                    {:else}
                        <span class="text-gray-400">-</span>
                    {/if}
				{:else}
					{row[column.key] ?? '-'}
				{/if}
			</svelte:fragment>
		</DataTable>
	</Card>
</div>
