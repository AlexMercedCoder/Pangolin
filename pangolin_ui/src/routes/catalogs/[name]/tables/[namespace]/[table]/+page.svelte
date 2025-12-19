<script lang="ts">
    import { onMount } from 'svelte';
    import { page } from '$app/stores';
    import { icebergApi, type Table } from '$lib/api/iceberg';
    import Card from '$lib/components/ui/Card.svelte';
    import Button from '$lib/components/ui/Button.svelte';
    import { notifications } from '$lib/stores/notifications';
    import { goto } from '$app/navigation';

    let loading = true;
    let table: Table | null = null;
    let error: string | null = null;
    
    // Params
    $: catalogName = $page.params.name;
    $: namespace = $page.params.namespace; // Assuming single level or handled via param matching (needs [...namespace] if multipart)
    $: tableName = $page.params.table;

    // Handle multipart namespaces if the route is defined as [name]/tables/[...namespace]/[table]
    // But for now let's assume simple route and I'll create the directory structure appropriately.
    // If I use `[namespace]` it catches one segment. If I use `[...namespace]`, it catches potentially the table too if nested incorrectly.
    // Let's use `tables/[...namespace]` where the LAST part is the table? No, that's ambiguous.
    // The previous href was `tables/${nsParts.join('.')}/${t.name}`.
    // This implies `namespace` param is a DOT SEPARATED STRING in one segment.
    // So `[namespace]` is correct as long as it handles the dots (which URL usually does).
    
    let activeTab = 'details';

    onMount(async () => {
        await loadTable();
    });

    async function loadTable() {
        if (!catalogName || !namespace || !tableName) return;
        
        loading = true; 
        try {
            // Namespace comes in as dot-separated string from URL
            const nsParts = namespace.split('.');
            table = await icebergApi.loadTable(catalogName, nsParts, tableName);
            console.log('Loaded table:', table);
        } catch (e: any) {
            console.error('Failed to load table:', e);
            error = e.message;
            notifications.error(`Failed to load table: ${e.message}`);
        } finally {
            loading = false;
        }
    }
</script>

<svelte:head>
    <title>{tableName || 'Asset'} - Pangolin</title>
</svelte:head>

<div class="space-y-6">
    <!-- Header -->
    <div class="flex items-center justify-between">
        <div>
            <div class="flex items-center gap-3">
                <button
                    on:click={() => history.back()}
                    class="text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white"
                >
                    ← Back
                </button>
                <h1 class="text-3xl font-bold text-gray-900 dark:text-white flex items-center gap-2">
                    <span class="material-icons text-2xl text-blue-500">
                        {table?.schemas?.length ? 'table_chart' : 'description'}
                    </span>
                    {tableName}
                </h1>
            </div>
            <div class="mt-2 text-gray-600 dark:text-gray-400 flex gap-2 text-sm">
                <span class="px-2 py-0.5 bg-gray-100 dark:bg-gray-800 rounded">Catalog: {catalogName}</span>
                <span class="px-2 py-0.5 bg-gray-100 dark:bg-gray-800 rounded">Namespace: {namespace}</span>
            </div>
        </div>
        <div class="flex items-center gap-3">
            <Button variant="outline" on:click={loadTable}>
                <span class="material-icons text-sm mr-1">refresh</span> Refresh
            </Button>
        </div>
    </div>

    <!-- Tabs -->
    <div class="border-b border-gray-200 dark:border-gray-700">
        <nav class="-mb-px flex space-x-8" aria-label="Tabs">
            <button
                class="{activeTab === 'details' ? 'border-primary-500 text-primary-600' : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'} whitespace-nowrap py-4 px-1 border-b-2 font-medium text-sm"
                on:click={() => activeTab = 'details'}
            >
                Details
            </button>
            <button
                class="{activeTab === 'data' ? 'border-primary-500 text-primary-600' : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'} whitespace-nowrap py-4 px-1 border-b-2 font-medium text-sm"
                on:click={() => activeTab = 'data'}
            >
                Data Preview
            </button>
        </nav>
    </div>

    {#if loading}
        <Card>
            <div class="flex items-center justify-center py-12">
                <div class="w-8 h-8 border-3 border-primary-600 border-t-transparent rounded-full animate-spin" />
            </div>
        </Card>
    {:else if error}
        <Card>
             <div class="text-center py-8">
                <span class="material-icons text-4xl text-red-500 mb-2">error_outline</span>
                <h3 class="text-lg font-medium text-gray-900 dark:text-white">Failed to load asset</h3>
                <p class="text-gray-500 mt-1">{error}</p>
                <Button variant="primary" class="mt-4" on:click={loadTable}>Retry</Button>
            </div>
        </Card>
    {:else if table}
        {#if activeTab === 'details'}
            <!-- Schema (Only if present) -->
            {#if table.schemas && table.schemas.length > 0}
                <Card title="Schema (Iceberg)">
                    <div class="overflow-x-auto">
                        <table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
                            <thead class="bg-gray-50 dark:bg-gray-800">
                                <tr>
                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Name</th>
                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Type</th>
                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Required</th>
                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Comment</th>
                                </tr>
                            </thead>
                            <tbody class="bg-white dark:bg-gray-900 divide-y divide-gray-200 dark:divide-gray-700">
                                 {#each table.schemas[0].fields as field}
                                     <tr>
                                         <td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900 dark:text-white">{field.name}</td>
                                         <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                                             <code class="bg-gray-100 dark:bg-gray-800 px-1 py-0.5 rounded text-xs">{typeof field.type === 'string' ? field.type : JSON.stringify(field.type)}</code>
                                         </td>
                                         <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                                             {#if field.required}
                                                 <span class="text-red-500 text-xs font-bold">YES</span>
                                             {:else}
                                                 <span class="text-gray-400 text-xs">NO</span>
                                             {/if}
                                         </td>
                                         <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">{field.doc || '-'}</td>
                                     </tr>
                                 {/each}
                            </tbody>
                        </table>
                    </div>
                </Card>
            {:else}
                 <Card title="Asset Details" class="bg-blue-50 dark:bg-blue-900/10 border-blue-100 dark:border-blue-800">
                    <p class="text-gray-600 dark:text-gray-400">
                        This asset does not have a standard Iceberg schema. It may be a generic file or view.
                    </p>
                 </Card>
            {/if}

            <!-- Metadata Properties -->
            <Card title="Properties" class="mt-6">
                 <dl class="grid grid-cols-1 md:grid-cols-2 gap-x-4 gap-y-4">
                    {#if table.properties && Object.keys(table.properties).length > 0}
                        {#each Object.entries(table.properties) as [key, value]}
                            <div class="sm:col-span-1">
                                <dt class="text-sm font-medium text-gray-500 dark:text-gray-400 truncate" title={key}>{key}</dt>
                                <dd class="mt-1 text-sm text-gray-900 dark:text-white break-all">{value}</dd>
                            </div>
                        {/each}
                    {:else}
                        <div class="col-span-2 text-gray-500 italic">No properties set</div>
                    {/if}
                 </dl>
            </Card>
            
            <!-- Snapshots Placeholder -->
            {#if table.snapshots && table.snapshots.length > 0}
            <Card title="History" class="mt-6">
                 <div class="text-sm text-gray-500">
                    <ul class="space-y-2">
                        {#each table.snapshots.slice(0, 5) as snap}
                            <li class="flex justify-between border-b border-gray-100 dark:border-gray-800 pb-2">
                                <span>ID: <span class="font-mono">{snap['snapshot-id']}</span></span>
                                <span class="text-gray-400">{new Date(snap['timestamp-ms']).toLocaleString()}</span>
                            </li>
                        {/each}
                    </ul>
                 </div>
            </Card>
            {/if}
        {:else if activeTab === 'data'}
            <Card title="Data Preview">
                <div class="text-center py-12">
                    <span class="material-icons text-4xl text-gray-300 mb-4">storage</span>
                    <h3 class="text-lg font-medium text-gray-900 dark:text-white">Query Engine Required</h3>
                    <p class="mt-2 text-gray-500 max-w-md mx-auto">
                        Pangolin is a Metadata Catalog. To view or query actual data rows, please connect a compute engine like PyIceberg, Dremio or Datafusion.
                    </p>
                    <div class="mt-6 bg-gray-50 dark:bg-gray-800 p-4 rounded text-left inline-block max-w-full overflow-x-auto">
                        <code class="text-sm font-mono text-gray-700 dark:text-gray-300">
                            # Example with PyIceberg<br/>
                            table = catalog.load_table("{namespace}.{tableName}")<br/>
                            df = table.scan().to_arrow()<br/>
                            print(df)
                        </code>
                    </div>
                </div>
            </Card>
        {/if}
    {/if}
</div>
