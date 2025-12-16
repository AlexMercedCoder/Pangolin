<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { catalogsApi, type Catalog } from '$lib/api/catalogs';
    import ExplorerNode from '$lib/components/explorer/ExplorerNode.svelte';
    import { triggerExplorerRefresh, refreshExplorer } from '$lib/stores/explorer';
    import { notifications } from '$lib/stores/notifications';

    let loading = true;
    let error: string | null = null;
    let nodes: any[] = [];
    let unsubscribe: () => void;

    async function loadCatalogs() {
        loading = true;
        error = null;
        try {
            const catalogs = await catalogsApi.list();
            nodes = catalogs.map(c => ({
                id: c.name,
                label: c.name,
                type: 'catalog',
                icon: '📚',
                hasChildren: true,
                expanded: false
            }));
        } catch (e: any) {
            console.error('Failed to load catalogs:', e);
            error = e.message || 'Failed to load catalogs';
            // Don't show notification on initial load to avoid clutter, just show inline error
        } finally {
            loading = false;
        }
    }

    onMount(() => {
        loadCatalogs();
        
        unsubscribe = refreshExplorer.subscribe(val => {
            if (val.timestamp > 0 && (val.type === 'catalog' || val.type === 'all')) {
                loadCatalogs();
            }
        });
    });

    onDestroy(() => {
        if (unsubscribe) unsubscribe();
    });
</script>

<div class="h-full flex flex-col">
    <div class="p-4 border-b border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800 flex justify-between items-center">
        <h2 class="text-sm font-semibold uppercase tracking-wider text-gray-500 dark:text-gray-400">
            Data Explorer
        </h2>
        <button 
            class="p-1 hover:bg-gray-200 dark:hover:bg-gray-700 rounded text-gray-500 transition-colors"
            title="Refresh"
            on:click={() => triggerExplorerRefresh('all')}
        >
            <span class="material-icons text-sm">refresh</span>
        </button>
    </div>
    
    <div class="flex-1 overflow-y-auto custom-scrollbar p-2 space-y-1 bg-white dark:bg-gray-900">
        {#if loading}
            <div class="flex justify-center p-4">
                <div class="animate-spin h-5 w-5 border-2 border-primary-600 border-t-transparent rounded-full"></div>
            </div>
        {:else if error}
            <div class="p-4 text-red-500 text-sm bg-red-50 dark:bg-red-900/10 rounded">
                {error}
                <button class="text-xs underline mt-2" on:click={loadCatalogs}>Retry</button>
            </div>
        {:else if nodes.length === 0}
            <div class="p-4 text-gray-500 text-sm text-center">
                No catalogs found.
            </div>
        {:else}
            {#each nodes as node (node.id)}
                <ExplorerNode {...node} />
            {/each}
        {/if}
    </div>
</div>
