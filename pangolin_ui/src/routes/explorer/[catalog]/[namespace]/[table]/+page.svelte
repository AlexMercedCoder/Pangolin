
<script lang="ts">
	import { page } from '$app/stores';
	import { icebergApi, type Table } from '$lib/api/iceberg';
    import TableDetail from '$lib/components/explorer/TableDetail.svelte';
    import RequestAccessModal from '$lib/components/discovery/RequestAccessModal.svelte';

	$: catalogName = $page.params.catalog || '';
    $: namespaceParam = $page.params.namespace || '';
    $: tableName = $page.params.table || '';
    $: namespaceParts = namespaceParam ? namespaceParam.split('.') : [];

    let table: Table | null = null;
    let loading = true;
    let error: string | null = null;
    let isForbidden = false;
    let showRequestModal = false;
    $: assetId = $page.url.searchParams.get('id');

    $: if (catalogName && namespaceParam && tableName) {
        loadTable();
    }

    async function loadTable() {
        loading = true;
        error = null;
        isForbidden = false;
        try {
            try {
                table = await icebergApi.loadTable(catalogName, namespaceParts, tableName);
            } catch (icebergError: any) {
                if (icebergError.status === 403 || icebergError.message?.includes('403') || icebergError.message?.includes('Forbidden')) {
                   // Propagate 403 immediately to outer catch
                   throw icebergError;
                }
                console.log("Failed to load as Iceberg table, attempting fallback to generic asset:", icebergError);
                // Fallback to generic asset
                table = await icebergApi.getAsset(catalogName, namespaceParts, tableName);
            }
        } catch (e: any) {
            console.error("Failed to load asset:", e);
            if (e.status === 403 || e.message?.includes('403') || e.message?.includes('Forbidden')) {
                 isForbidden = true;
                 error = "You do not have permission to view this asset.";
            } else {
                 error = e.message || "Failed to load asset";
            }
        } finally {
            loading = false;
        }
    }
</script>

<div class="h-full flex flex-col">
    <div class="p-6 border-b border-gray-200 dark:border-gray-700">
        <div class="flex items-center gap-2 text-sm text-gray-500 dark:text-gray-400 mb-2">
            <a href="/explorer/{encodeURIComponent(catalogName)}" class="hover:text-primary-600 hover:underline">
                {catalogName}
            </a>
            {#each namespaceParts as part, i}
                <span>/</span>
                <a 
                    href="/explorer/{encodeURIComponent(catalogName)}/{encodeURIComponent(namespaceParts.slice(0, i+1).join('.'))}"
                    class="hover:text-primary-600 hover:underline"
                >
                    {part}
                </a>
            {/each}
            <span>/</span>
            <span class="text-gray-900 dark:text-white font-medium">{tableName}</span>
        </div>
        <div class="flex items-center gap-3">
             <span class="text-2xl">📄</span>
             <h1 class="text-2xl font-bold text-gray-900 dark:text-white">
                {tableName}
            </h1>
        </div>
    </div>

    <div class="flex-1 overflow-y-auto p-6">
        {#if loading}
            <div class="flex justify-center">
                <div class="animate-spin h-8 w-8 border-4 border-primary-600 border-t-transparent rounded-full"></div>
            </div>
        {:else if error}
            <div class="flex flex-col items-center justify-center p-12 text-center">
                {#if isForbidden}
                    <div class="bg-yellow-50 dark:bg-yellow-900/20 p-6 rounded-lg max-w-lg">
                        <h3 class="text-xl font-bold text-gray-900 dark:text-white mb-2">Access Denied</h3>
                        <p class="text-gray-600 dark:text-gray-300 mb-6">{error}</p>
                        
                        {#if assetId}
                             <button
                                on:click={() => showRequestModal = true}
                                class="px-6 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 font-medium"
                             >
                                Request Access
                             </button>
                        {:else}
                            <p class="text-xs text-gray-500">Asset ID missing, cannot request access.</p>
                        {/if}
                    </div>
                {:else}
                    <div class="bg-red-50 dark:bg-red-900/20 text-red-600 dark:text-red-400 p-4 rounded-lg">
                        {error}
                    </div>
                {/if}
            </div>
        {:else if table}
            <TableDetail {table} />
        {/if}
    </div>
    </div>
</div>

<RequestAccessModal 
    bind:open={showRequestModal} 
    assetId={assetId} 
    assetName={tableName} 
/>
