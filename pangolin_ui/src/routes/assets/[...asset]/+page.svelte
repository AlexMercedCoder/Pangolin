<script lang="ts">
    import { onMount } from 'svelte';
    import { page } from '$app/stores';
    import { authStore } from '$lib/stores/auth';
    
    $: token = $authStore.token;
    $: user = $authStore.user;
    import Card from '$lib/components/ui/Card.svelte';
    import type { Asset } from '$lib/api/iceberg';
    import SchemaField from '$lib/components/explorer/SchemaField.svelte';

    // Extract from URL or use defaults
    let catalogName = $page.params.catalog || "default";
    let branchName = ($page.params as any).branch || "main";
    let assetPath = $page.params.asset || "";
    
    let asset: Asset | null = null;
    let error = "";
    let activeTab = "schema"; // schema | properties

    onMount(async () => {
        if (!user) return;
        
        // Construct API path: /v1/{prefix}/namespaces/{namespace}/tables/{table}
        // assetPath is "namespace/table" or "ns1/ns2/table"
        // We need to split it.
        const parts = assetPath.split('/');
        const tableName = parts.pop();
        const namespace = parts.join('\x1F'); // Iceberg REST uses \x1F separator? Or we use slash in our UI path?
        // Our UI path is /assets/ns1/ns2/table
        // SvelteKit [...asset] gives "ns1/ns2/table" string.
        
        // Let's assume standard slash separator for namespace in UI, but API expects encoded namespace?
        // Our API `list_tables` returns names. `load_table` expects namespace and table.
        // Let's try to fetch table metadata.
        
        const res = await fetch(`/v1/${catalogName}/namespaces/${namespace}/tables/${tableName}`, {
            headers: { 'Authorization': `Bearer ${token}` }
        });
        
        if (res.ok) {
            asset = await res.json();
        } else {
            error = "Failed to load asset details";
        }
    });
</script>

<div class="p-6">
    <div class="flex items-center mb-6">
        <a href="/" class="text-blue-400 hover:text-blue-300 mr-4">&larr; Back to Dashboard</a>
        <h1 class="text-2xl font-bold">{assetPath}</h1>
    </div>

    {#if error}
        <p class="text-red-500 mb-4">{error}</p>
    {:else if asset}
        <div class="bg-gray-800 rounded p-6">
            <div class="flex border-b border-gray-700 mb-6">
                <button 
                    class="px-4 py-2 font-medium {activeTab === 'schema' ? 'text-blue-400 border-b-2 border-blue-400' : 'text-gray-400 hover:text-white'}"
                    on:click={() => activeTab = 'schema'}
                >
                    Schema
                </button>
                <button 
                    class="px-4 py-2 font-medium {activeTab === 'properties' ? 'text-blue-400 border-b-2 border-blue-400' : 'text-gray-400 hover:text-white'}"
                    on:click={() => activeTab = 'properties'}
                >
                    Properties
                </button>
            </div>

            {#if activeTab === 'schema'}
                <div class="space-y-2">
                    {#if asset.schema && asset.schema.fields}
                        <div class="bg-gray-900 p-4 rounded">
                            {#each asset.schema.fields as field}
                                <SchemaField {field} depth={0} />
                            {/each}
                        </div>
                    {:else}
                        <p class="text-gray-400 italic">No schema available</p>
                    {/if}
                </div>
            {:else}
                <div class="grid grid-cols-1 gap-4">
                    {#each Object.entries(asset.properties || {}) as [key, value]}
                        <div class="flex justify-between border-b border-gray-700 py-2">
                            <span class="text-gray-400">{key}</span>
                            <span class="font-mono text-sm">{value}</span>
                        </div>
                    {/each}
                </div>
            {/if}
        </div>
    {:else}
        <p class="text-gray-400">Loading...</p>
    {/if}
</div>
