<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { icebergApi, type TableIdentifier, type CreateNamespaceRequest, type CreateTableRequest } from '$lib/api/iceberg';
    import Button from '$lib/components/ui/Button.svelte';
    import CreateNamespaceDialog from '$lib/components/explorer/CreateNamespaceDialog.svelte';
    import CreateTableDialog from '$lib/components/explorer/CreateTableDialog.svelte';
    import { notifications } from '$lib/stores/notifications';

	$: catalogName = $page.params.catalog || '';
    $: namespaceParam = $page.params.namespace || '';
    $: namespaceParts = namespaceParam ? namespaceParam.split('.') : [];

    let namespaces: string[][] = [];
    let tables: TableIdentifier[] = [];
    let loading = true;
    let error: string | null = null;

    // Creation State
    let showCreateNamespace = false;
    let showCreateTable = false;
    let creating = false;

    $: if (catalogName && namespaceParam) {
        loadNamespace();
    }

    async function loadNamespace() {
        loading = true;
        error = null;
        try {
            const [nsList, tblList] = await Promise.all([
                icebergApi.listNamespaces(catalogName, namespaceParts),
                icebergApi.listTables(catalogName, namespaceParts)
            ]);
            namespaces = nsList;
            tables = tblList;
        } catch (e: any) {
            error = e.message;
        } finally {
            loading = false;
        }
    }

    import { triggerExplorerRefresh } from '$lib/stores/explorer';

    async function handleCreateNamespace(event: CustomEvent) {
        creating = true;
        try {
            const newName = event.detail.name;
            // Support dot notation in name input by splitting
            const newParts = newName.split('.');
            // Combine current namespace parts with new parts
            const fullNamespace = [...namespaceParts, ...newParts];
            
            const req: CreateNamespaceRequest = {
                namespace: fullNamespace
            };

            await icebergApi.createNamespace(catalogName, req);
            notifications.success(`Namespace "${newName}" created`);
            showCreateNamespace = false;
            triggerExplorerRefresh('content'); // Refresh sidebar content
            loadNamespace(); // Refresh current view
        } catch (e: any) {
            console.error(e);
            notifications.error(`Failed to create namespace: ${e.message}`);
        } finally {
            creating = false;
        }
    }

    async function handleCreateTable(event: CustomEvent) {
        creating = true;
        try {
            const req: CreateTableRequest = event.detail;
            await icebergApi.createTable(catalogName, namespaceParts, req);
            notifications.success(`Table "${req.name}" created`);
            showCreateTable = false;
            triggerExplorerRefresh('content'); // Refresh sidebar content
            loadNamespace(); // Refresh current view
        } catch (e: any) {
            console.error(e);
            notifications.error(`Failed to create table: ${e.message}`);
        } finally {
            creating = false;
        }
    }
</script>

<div class="h-full flex flex-col">
    <div class="p-6 border-b border-gray-200 dark:border-gray-700 flex justify-between items-start">
        <div>
            <div class="flex items-center gap-2 text-sm text-gray-500 dark:text-gray-400 mb-2">
                <a href="/explorer/{encodeURIComponent(catalogName)}" class="hover:text-primary-600 hover:underline">
                    {catalogName}
                </a>
                {#each namespaceParts as part, i}
                    <span>/</span>
                    <span class="{i === namespaceParts.length - 1 ? 'text-gray-900 dark:text-white font-medium' : ''}">
                        {part}
                    </span>
                {/each}
            </div>
            <h1 class="text-2xl font-bold text-gray-900 dark:text-white">
                {namespaceParts[namespaceParts.length - 1]}
            </h1>
        </div>
        <Button size="sm" on:click={() => showCreateNamespace = true}>
            Create Namespace
        </Button>
    </div>

    <div class="flex-1 overflow-y-auto p-6 space-y-8">
        {#if loading}
            <div class="flex justify-center">
                <div class="animate-spin h-8 w-8 border-4 border-primary-600 border-t-transparent rounded-full"></div>
            </div>
        {:else if error}
            <div class="bg-red-50 dark:bg-red-900/20 text-red-600 dark:text-red-400 p-4 rounded-lg">
                {error}
            </div>
        {:else}
            <!-- Sub-Namespaces -->
            {#if namespaces.length > 0}
                <section>
                    <h3 class="text-sm font-semibold uppercase tracking-wider text-gray-500 dark:text-gray-400 mb-4">
                        Namespaces
                    </h3>
                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                        {#each namespaces as ns}
                            <button 
                                class="p-4 bg-gray-50 dark:bg-gray-800/50 rounded-xl border border-gray-200 dark:border-gray-700 hover:border-primary-500 dark:hover:border-primary-500 transition-colors text-left group"
                                on:click={() => goto(`/explorer/${encodeURIComponent(catalogName)}/${encodeURIComponent(ns.join('.'))}`)}
                            >
                                <div class="flex items-center gap-3">
                                    <span class="text-2xl group-hover:scale-110 transition-transform">📦</span>
                                    <div class="font-medium text-gray-900 dark:text-white">
                                        {ns[ns.length - 1]}
                                    </div>
                                </div>
                            </button>
                        {/each}
                    </div>
                </section>
            {/if}

            <!-- Tables -->
            <section>
                <div class="flex items-center justify-between mb-4">
                    <h3 class="text-sm font-semibold uppercase tracking-wider text-gray-500 dark:text-gray-400">
                        Tables
                    </h3>
                    {#if namespaces.length === 0 && tables.length === 0}
                         <!-- Empty state handled below -->
                    {:else}
                         <Button size="sm" variant="ghost" on:click={() => showCreateTable = true}>+ Create Table</Button>
                    {/if}
                </div>
                
                {#if tables.length > 0}
                    <div class="border border-gray-200 dark:border-gray-700 rounded-lg overflow-hidden">
                        <table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
                             <thead class="bg-gray-50 dark:bg-gray-800">
                                <tr>
                                    <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Name</th>
                                    <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Identifier</th>
                                </tr>
                             </thead>
                             <tbody class="bg-white dark:bg-gray-900 divide-y divide-gray-200 dark:divide-gray-700">
                                {#each tables as table}
                                    <tr 
                                        class="hover:bg-gray-50 dark:hover:bg-gray-800 cursor-pointer transition-colors"
                                        on:click={() => goto(`/explorer/${encodeURIComponent(catalogName)}/${encodeURIComponent(namespaceParam)}/${encodeURIComponent(table.name)}`)}
                                    >
                                        <td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900 dark:text-white flex items-center gap-2">
                                            <span>📄</span>
                                            {table.name}
                                        </td>
                                        <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400 font-mono">
                                            {table.namespace.join('.')}.{table.name}
                                        </td>
                                    </tr>
                                {/each}
                             </tbody>
                        </table>
                    </div>
                {:else if namespaces.length === 0}
                    <div class="text-center py-12 bg-gray-50 dark:bg-gray-800/50 rounded-lg border border-dashed border-gray-300 dark:border-gray-700">
                        <p class="text-gray-500 dark:text-gray-400">This namespace is empty.</p>
                        <div class="mt-4 flex justify-center gap-2">
                             <Button size="sm" variant="secondary" on:click={() => showCreateNamespace = true}>Create Sub-Namespace</Button>
                             <Button size="sm" on:click={() => showCreateTable = true}>Create Table</Button>
                        </div>
                    </div>
                {/if}
            </section>
        {/if}
    </div>

    <!-- Dialogs -->
    <CreateNamespaceDialog 
        bind:open={showCreateNamespace}
        bind:loading={creating}
        on:create={handleCreateNamespace}
    />
    
    <CreateTableDialog
        bind:open={showCreateTable}
        bind:loading={creating}
        on:create={handleCreateTable}
    />
</div>
