<script lang="ts">
    import type { Table } from '$lib/api/iceberg';
    import Card from '$lib/components/ui/Card.svelte';

    export let table: Table;

    // Helper to get schema fields safely
    $: schema = table.schemas?.[0] || {}; // Use current schema (usually id matches current_schema_id)
    $: fields = schema.fields || [];
    $: snapshots = table.snapshots || [];
    $: history = table.history || [];
    $: properties = table.properties || {};

    let activeTab: 'schema' | 'snapshots' | 'metadata' = 'schema';
</script>

<div class="space-y-6">
    <!-- Header info -->
    <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
         <Card>
             <div class="text-sm text-gray-500 dark:text-gray-400">Current Snapshot</div>
             <div class="font-mono text-lg font-medium text-gray-900 dark:text-white truncate">
                 {table.current_snapshot_id || 'None'}
             </div>
         </Card>
         <Card>
            <div class="text-sm text-gray-500 dark:text-gray-400">Format Version</div>
            <div class="font-medium text-gray-900 dark:text-white">
                v{table['format-version'] || '1'}
            </div>
        </Card>
        <Card>
            <div class="text-sm text-gray-500 dark:text-gray-400">Location</div>
            <div class="font-mono text-xs text-gray-900 dark:text-white break-all mt-1">
                {table.location || '-'}
            </div>
        </Card>
    </div>

    <!-- Tabs -->
    <div class="border-b border-gray-200 dark:border-gray-700">
        <nav class="-mb-px flex space-x-8" aria-label="Tabs">
            <button
                class="{activeTab === 'schema' ? 'border-primary-500 text-primary-600 dark:text-primary-400' : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300 dark:text-gray-400 dark:hover:text-gray-300'} whitespace-nowrap py-4 px-1 border-b-2 font-medium text-sm"
                on:click={() => activeTab = 'schema'}
            >
                Schema ({fields.length})
            </button>
            <button
                class="{activeTab === 'snapshots' ? 'border-primary-500 text-primary-600 dark:text-primary-400' : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300 dark:text-gray-400 dark:hover:text-gray-300'} whitespace-nowrap py-4 px-1 border-b-2 font-medium text-sm"
                on:click={() => activeTab = 'snapshots'}
            >
                Snapshots ({snapshots.length})
            </button>
            <button
                class="{activeTab === 'metadata' ? 'border-primary-500 text-primary-600 dark:text-primary-400' : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300 dark:text-gray-400 dark:hover:text-gray-300'} whitespace-nowrap py-4 px-1 border-b-2 font-medium text-sm"
                on:click={() => activeTab = 'metadata'}
            >
                Properties
            </button>
        </nav>
    </div>

    <!-- Content -->
    {#if activeTab === 'schema'}
        <Card>
            <div class="overflow-x-auto">
                <table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
                    <thead class="bg-gray-50 dark:bg-gray-800">
                        <tr>
                            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">ID</th>
                            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Name</th>
                            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Type</th>
                            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Required</th>
                            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Doc</th>
                        </tr>
                    </thead>
                    <tbody class="bg-white dark:bg-gray-900 divide-y divide-gray-200 dark:divide-gray-700">
                        {#each fields as field}
                            <tr>
                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">{field.id}</td>
                                <td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900 dark:text-white">{field.name}</td>
                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400 font-mono">{typeof field.type === 'string' ? field.type : JSON.stringify(field.type)}</td>
                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                                    {#if field.required}
                                        <span class="text-red-500 font-bold">Yes</span>
                                    {:else}
                                        <span class="text-gray-400">No</span>
                                    {/if}
                                </td>
                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400 text-xs italic">{field.doc || '-'}</td>
                            </tr>
                        {/each}
                    </tbody>
                </table>
            </div>
        </Card>
    {:else if activeTab === 'snapshots'}
        <Card>
            <div class="overflow-x-auto">
                <table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
                    <thead class="bg-gray-50 dark:bg-gray-800">
                         <tr>
                             <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">ID</th>
                             <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Timestamp</th>
                             <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Operation</th>
                             <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Summary</th>
                         </tr>
                    </thead>
                    <tbody class="bg-white dark:bg-gray-900 divide-y divide-gray-200 dark:divide-gray-700">
                        {#each [...snapshots].reverse() as snapshot}
                            <tr>
                                <td class="px-6 py-4 whitespace-nowrap text-sm font-mono text-gray-900 dark:text-white">{snapshot['snapshot-id']}</td>
                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                                    {new Date(snapshot['timestamp-ms']).toLocaleString()}
                                </td>
                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900 dark:text-white capitalize">
                                    {snapshot.summary?.operation || '-'}
                                </td>
                                <td class="px-6 py-4 text-sm text-gray-500 dark:text-gray-400 max-w-xs truncate">
                                    {JSON.stringify(snapshot.summary)}
                                </td>
                            </tr>
                        {/each}
                        {#if snapshots.length === 0}
                            <tr>
                                <td colspan="4" class="px-6 py-8 text-center text-gray-500">No snapshots found</td>
                            </tr>
                        {/if}
                    </tbody>
                </table>
            </div>
        </Card>
    {:else if activeTab === 'metadata'}
        <Card>
             <dl class="space-y-3">
                 {#each Object.entries(properties) as [key, value]}
                     <div class="grid grid-cols-1 md:grid-cols-3 gap-2">
                         <dt class="text-sm font-medium text-gray-500 dark:text-gray-400 break-all">{key}</dt>
                         <dd class="md:col-span-2 text-sm text-gray-900 dark:text-white break-all font-mono bg-gray-50 dark:bg-gray-800 p-2 rounded">
                             {value}
                         </dd>
                     </div>
                 {/each}
                 {#if Object.keys(properties).length === 0}
                     <div class="text-center text-gray-500 italic">No properties set</div>
                 {/if}
             </dl>
        </Card>
    {/if}
</div>
