<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { icebergApi, type CreateNamespaceRequest } from '$lib/api/iceberg';
    import Button from '$lib/components/ui/Button.svelte';
    import CreateNamespaceDialog from '$lib/components/explorer/CreateNamespaceDialog.svelte';
    import { notifications } from '$lib/stores/notifications';

	$: catalogName = $page.params.catalog || '';

    let namespaces: string[][] = [];
    let loading = true;
    let error: string | null = null;
    let showCreateNamespace = false;
    let creating = false;

    $: if (catalogName) {
        loadCatalog();
    }

    async function loadCatalog() {
        loading = true;
        error = null;
        try {
            namespaces = await icebergApi.listNamespaces(catalogName);
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
            const fullNamespace = newName.split('.');
            
            const req: CreateNamespaceRequest = {
                namespace: fullNamespace
            };

            await icebergApi.createNamespace(catalogName, req);
            notifications.success(`Namespace "${newName}" created`);
            showCreateNamespace = false;
            triggerExplorerRefresh('content'); // Refresh sidebar content
            loadCatalog(); // Refresh current view
        } catch (e: any) {
            console.error(e);
            notifications.error(`Failed to create namespace: ${e.message}`);
        } finally {
            creating = false;
        }
    }
</script>

<div class="h-full flex flex-col">
    <div class="p-6 border-b border-gray-200 dark:border-gray-700 flex justify-between items-start">
        <div>
            <div class="flex items-center gap-2 text-sm text-gray-500 dark:text-gray-400 mb-2">
                <span>Explorer</span>
                <span>/</span>
                <span class="text-gray-900 dark:text-white font-medium">{catalogName}</span>
            </div>
            <h1 class="text-2xl font-bold text-gray-900 dark:text-white">
                {catalogName}
            </h1>
        </div>
        <Button size="sm" on:click={() => showCreateNamespace = true}>
            Create Namespace
        </Button>
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
        {:else}
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                {#each namespaces as ns}
                    <button 
                        class="p-4 bg-gray-50 dark:bg-gray-800/50 rounded-xl border border-gray-200 dark:border-gray-700 hover:border-primary-500 dark:hover:border-primary-500 transition-colors text-left group"
                        on:click={() => goto(`/explorer/${encodeURIComponent(catalogName)}/${encodeURIComponent(ns.join('.'))}`)}
                    >
                        <div class="flex items-center gap-3">
                            <span class="text-2xl group-hover:scale-110 transition-transform">📦</span>
                            <div>
                                <div class="font-medium text-gray-900 dark:text-white">
                                    {ns[ns.length - 1]}
                                </div>
                                <div class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
                                    Namespace
                                </div>
                            </div>
                        </div>
                    </button>
                {/each}
                {#if namespaces.length === 0}
                    <div class="col-span-full text-center py-12 bg-gray-50 dark:bg-gray-800/50 rounded-lg border border-dashed border-gray-300 dark:border-gray-700">
                        <p class="text-gray-500 dark:text-gray-400 mb-4">This catalog is empty.</p>
                        <Button size="sm" variant="secondary" on:click={() => showCreateNamespace = true}>
                            Create Namespace
                        </Button>
                    </div>
                {/if}
            </div>
        {/if}
    </div>

    <!-- Create Namespace Dialog -->
    <CreateNamespaceDialog 
        bind:open={showCreateNamespace}
        bind:loading={creating}
        on:create={handleCreateNamespace}
    />
</div>

