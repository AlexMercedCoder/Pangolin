```svelte
<script lang="ts">
	import { page } from '$app/stores';
	import { icebergApi, type Table } from '$lib/api/iceberg';
    import TableDetail from '$lib/components/explorer/TableDetail.svelte';

	$: catalogName = $page.params.catalog || '';
    $: namespaceParam = $page.params.namespace || '';
    $: tableName = $page.params.table || '';
    $: namespaceParts = namespaceParam ? namespaceParam.split('.') : [];

    let table: Table | null = null;
    let loading = true;
    let error: string | null = null;

    $: if (catalogName && namespaceParam && tableName) {
        loadTable();
    }

    async function loadTable() {
        loading = true;
        error = null;
        try {
            table = await icebergApi.loadTable(catalogName, namespaceParts, tableName);
        } catch (e: any) {
            error = e.message;
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
            <div class="bg-red-50 dark:bg-red-900/20 text-red-600 dark:text-red-400 p-4 rounded-lg">
                {error}
            </div>
        {:else if table}
            <TableDetail {table} />
        {/if}
    </div>
</div>
