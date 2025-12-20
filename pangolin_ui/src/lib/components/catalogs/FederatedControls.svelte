<script lang="ts">
    import { onMount } from 'svelte';
    import Button from '$lib/components/ui/Button.svelte';
    import Card from '$lib/components/ui/Card.svelte';
    import { catalogsApi, type SyncStats } from '$lib/api/catalogs';
    import { notifications } from '$lib/stores/notifications';

    import { createEventDispatcher } from 'svelte';

    export let catalogName: string;

    const dispatch = createEventDispatcher();

    let stats: SyncStats | null = null;
    let loading = false;
    let syncing = false;

    async function loadStats() {
        loading = true;
        try {
            stats = await catalogsApi.getStats(catalogName);
        } catch (e: any) {
            console.error('Failed to load sync stats:', e);
            notifications.error('Failed to load sync stats');
        } finally {
            loading = false;
        }
    }

    async function handleSync() {
        syncing = true;
        try {
            await catalogsApi.sync(catalogName);
            notifications.success('Sync triggered successfully');
            dispatch('sync'); // Notify parent to reload data
            // Poll for update or just reload stats after a delay
            setTimeout(loadStats, 1000); 
        } catch (e: any) {
            console.error('Sync failed:', e);
            notifications.error(`Sync failed: ${e.message}`);
        } finally {
            syncing = false;
        }
    }

    onMount(() => {
        loadStats();
    });
</script>

<Card>
    <div class="flex items-center justify-between mb-4">
        <h3 class="text-lg font-semibold text-gray-900 dark:text-white flex items-center gap-2">
            <span class="material-icons text-blue-500">sync_alt</span>
            Federated Sync
        </h3>
        <Button 
            size="sm" 
            variant="primary" 
            on:click={handleSync} 
            loading={syncing}
            disabled={loading || syncing}
        >
            Sync Now
        </Button>
    </div>

    {#if loading && !stats}
        <div class="animate-pulse space-y-2">
            <div class="h-4 bg-gray-200 dark:bg-gray-700 rounded w-1/4"></div>
            <div class="h-4 bg-gray-200 dark:bg-gray-700 rounded w-1/2"></div>
        </div>
    {:else if stats}
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div class="bg-gray-50 dark:bg-gray-800 p-3 rounded-lg border border-gray-200 dark:border-gray-700">
                <div class="text-sm text-gray-500 dark:text-gray-400">Status</div>
                <div class="font-medium flex items-center gap-2 mt-1">
                    {#if stats.sync_status === 'Success'}
                        <span class="w-2 h-2 rounded-full bg-green-500"></span>
                        <span class="text-green-700 dark:text-green-400">Synced</span>
                    {:else if stats.sync_status === 'Syncing'}
                        <span class="w-2 h-2 rounded-full bg-blue-500 animate-pulse"></span>
                        <span class="text-blue-700 dark:text-blue-400">Syncing...</span>
                    {:else}
                         <span class="w-2 h-2 rounded-full bg-red-500"></span>
                        <span class="text-red-700 dark:text-red-400">{stats.sync_status}</span>
                    {/if}
                </div>
            </div>

            <div class="bg-gray-50 dark:bg-gray-800 p-3 rounded-lg border border-gray-200 dark:border-gray-700">
                <div class="text-sm text-gray-500 dark:text-gray-400">Last Synced</div>
                <div class="font-medium text-gray-900 dark:text-white mt-1">
                    {stats.last_synced_at ? new Date(stats.last_synced_at).toLocaleString() : 'Never'}
                </div>
            </div>

            <div class="bg-gray-50 dark:bg-gray-800 p-3 rounded-lg border border-gray-200 dark:border-gray-700">
                <div class="text-sm text-gray-500 dark:text-gray-400">Namespaces</div>
                <div class="font-medium text-gray-900 dark:text-white mt-1">
                    {stats.namespaces_synced}
                </div>
            </div>

            <div class="bg-gray-50 dark:bg-gray-800 p-3 rounded-lg border border-gray-200 dark:border-gray-700">
                <div class="text-sm text-gray-500 dark:text-gray-400">Tables</div>
                <div class="font-medium text-gray-900 dark:text-white mt-1">
                    {stats.tables_synced}
                </div>
            </div>
        </div>

        {#if stats.error_message}
            <div class="mt-4 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded text-sm text-red-700 dark:text-red-300">
                <strong>Error:</strong> {stats.error_message}
            </div>
        {/if}
    {:else}
        <p class="text-gray-500 italic">No sync stats available.</p>
    {/if}
</Card>
