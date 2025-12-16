<script lang="ts">
    import type { Table } from '$lib/api/iceberg';
    import Card from '$lib/components/ui/Card.svelte';
    import { onMount } from 'svelte';
    import { authStore } from '$lib/stores/auth';

    export let table: Table;
    export let assetId: string | undefined = undefined;

    // Helper to get schema fields safely
    $: schema = table.schemas?.[0] || {};
    $: fields = schema.fields || [];
    $: snapshots = table.snapshots || [];
    $: history = table.history || [];
    $: properties = table.properties || {};

    let activeTab: 'schema' | 'snapshots' | 'metadata' | 'business' = 'schema';
    
    // Business Metadata state
    let businessMetadata: any = null;
    let loadingMetadata = false;
    let editMode = false;
    let description = '';
    let tags: string[] = [];
    let discoverable = false;
    let newTag = '';

    $: canEdit = $authStore.user?.role === 'tenant-admin' || $authStore.user?.role === 'root';

    onMount(async () => {
        if (assetId) {
            await loadMetadata();
        }
    });

    async function loadMetadata() {
        if (!assetId) return;
        loadingMetadata = true;
        try {
            const res = await fetch(`/api/v1/business-metadata/${assetId}`, {
                headers: { 'Authorization': `Bearer ${$authStore.token}` }
            });
            if (res.ok) {
                const data = await res.json();
                businessMetadata = data.metadata;
                description = businessMetadata?.description || '';
                tags = businessMetadata?.tags || [];
                discoverable = businessMetadata?.discoverable || false;
            }
        } catch (e) {
            console.error('Failed to load metadata:', e);
        } finally {
            loadingMetadata = false;
        }
    }

    async function saveMetadata() {
        if (!assetId) return;
        try {
            const res = await fetch(`/api/v1/business-metadata/${assetId}`, {
                method: 'POST',
                headers: {
                    'Authorization': `Bearer ${$authStore.token}`,
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({ description, tags, discoverable, properties: {} })
            });
            if (res.ok) {
                await loadMetadata();
                editMode = false;
            } else {
                alert('Failed to save metadata: ' + await res.text());
            }
        } catch (e) {
            alert('Failed to save metadata');
        }
    }

    function addTag() {
        if (newTag && !tags.includes(newTag)) {
            tags = [...tags, newTag];
            newTag = '';
        }
    }

    function removeTag(tag: string) {
        tags = tags.filter(t => t !== tag);
    }

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
            <button
                class="{activeTab === 'business' ? 'border-primary-500 text-primary-600 dark:text-primary-400' : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300 dark:text-gray-400 dark:hover:text-gray-300'} whitespace-nowrap py-4 px-1 border-b-2 font-medium text-sm"
                on:click={() => activeTab = 'business'}
            >
                Business Info
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
    {:else if activeTab === 'business'}
        <Card>
            {#if loadingMetadata}
                <div class="text-center py-8 text-gray-500">Loading...</div>
            {:else if editMode}
                <div class="space-y-4">
                    <h3 class="text-lg font-medium">Edit Business Metadata</h3>
                    <div>
                        <label class="block text-sm font-medium mb-1">Description</label>
                        <textarea bind:value={description} class="w-full px-3 py-2 border rounded-md dark:bg-gray-800" rows="3" />
                    </div>
                    <div>
                        <label class="block text-sm font-medium mb-1">Tags</label>
                        <div class="flex gap-2 mb-2">
                            <input bind:value={newTag} on:keydown={(e) => e.key === 'Enter' && addTag()} class="flex-1 px-3 py-2 border rounded-md dark:bg-gray-800" placeholder="Add tag..." />
                            <button on:click={addTag} class="px-4 py-2 bg-gray-200 dark:bg-gray-700 rounded hover:bg-gray-300">Add</button>
                        </div>
                        <div class="flex flex-wrap gap-2">
                            {#each tags as tag}
                                <span class="inline-flex items-center gap-1 px-2 py-1 bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 rounded text-sm">
                                    {tag}
                                    <button on:click={() => removeTag(tag)} class="hover:text-blue-600">×</button>
                                </span>
                            {/each}
                        </div>
                    </div>
                    {#if canEdit}
                        <label class="flex items-center gap-2">
                            <input type="checkbox" bind:checked={discoverable} class="rounded" />
                            <span class="text-sm">Make discoverable to other users</span>
                        </label>
                    {/if}
                    <div class="flex gap-2">
                        <button on:click={saveMetadata} class="px-4 py-2 bg-primary-600 text-white rounded hover:bg-primary-700">Save</button>
                        <button on:click={() => { editMode = false; loadMetadata(); }} class="px-4 py-2 bg-gray-200 dark:bg-gray-700 rounded">Cancel</button>
                    </div>
                </div>
            {:else}
                <div class="space-y-4">
                    <div class="flex justify-between items-center">
                        <h3 class="text-lg font-medium">Business Metadata</h3>
                        {#if canEdit}
                            <button on:click={() => editMode = true} class="px-3 py-1 bg-primary-600 text-white rounded hover:bg-primary-700">Edit</button>
                        {/if}
                    </div>
                    <div>
                        <div class="text-sm font-medium text-gray-500 dark:text-gray-400">Description</div>
                        <div class="mt-1">{businessMetadata?.description || 'No description'}</div>
                    </div>
                    <div>
                        <div class="text-sm font-medium text-gray-500 dark:text-gray-400">Tags</div>
                        <div class="mt-1 flex flex-wrap gap-2">
                            {#if businessMetadata?.tags?.length > 0}
                                {#each businessMetadata.tags as tag}
                                    <span class="px-2 py-1 bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 rounded text-sm">{tag}</span>
                                {/each}
                            {:else}
                                <span class="text-gray-500">No tags</span>
                            {/if}
                        </div>
                    </div>
                    <div>
                        <div class="text-sm font-medium text-gray-500 dark:text-gray-400">Discoverable</div>
                        <div class="mt-1">{businessMetadata?.discoverable ? 'Yes' : 'No'}</div>
                    </div>
                </div>
            {/if}
        </Card>
    {/if}
</div>
