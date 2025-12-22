<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { catalogsApi, type Catalog } from '$lib/api/catalogs';
    import { icebergApi } from '$lib/api/iceberg';
    import ExplorerNode from '$lib/components/explorer/ExplorerNode.svelte';
    import CreateNamespaceDialog from '$lib/components/explorer/CreateNamespaceDialog.svelte';
    import CreateTableDialog from '$lib/components/explorer/CreateTableDialog.svelte';
    import { triggerExplorerRefresh, refreshExplorer } from '$lib/stores/explorer';
    import { notifications } from '$lib/stores/notifications';

    let loading = true;
    let error: string | null = null;
    let nodes: any[] = [];
    let unsubscribe: () => void;

    // Context Menu State
    let contextMenu = {
        visible: false,
        x: 0,
        y: 0,
        node: null as any
    };

    // Dialog State
    let showCreateNamespace = false;
    let showCreateTable = false;
    let creating = false;

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
                expanded: false,
                // Pass loader for namespaces
                loadChildren: () => loadNamespaces(c.name)
            }));
        } catch (e: any) {
            console.error('Failed to load catalogs:', e);
            error = e.message || 'Failed to load catalogs';
        } finally {
            loading = false;
        }
    }

    async function loadNamespaces(catalogName: string) {
        try {
            const nss = await icebergApi.listNamespaces(catalogName);
            // Namespace is array of strings (parts).
            return nss.map(nsParts => {
                const label = nsParts.join('.');
                return {
                    id: `${catalogName}.${label}`,
                    label: label,
                    type: 'namespace',
                    icon: '📁',
                    href: `/explorer/${catalogName}/${label}`, // Context for click
                    hasChildren: true,
                    expanded: false,
                    // Pass loader for tables
                    loadChildren: () => loadTables(catalogName, nsParts)
                };
            });
        } catch (e) {
            console.error(`Failed to load namespaces for ${catalogName}:`, e);
            return [];
        }
    }

    async function loadTables(catalogName: string, namespaceParts: string[]) {
        try {
            const tables = await icebergApi.listTables(catalogName, namespaceParts);
            return tables.map(t => ({
                id: `${catalogName}.${namespaceParts.join('.')}.${t.name}`,
                label: t.name,
                type: 'table',
                icon: '📄',
                href: `/explorer/${catalogName}/${namespaceParts.join('.')}/${t.name}`,
                hasChildren: false,
                expanded: false
            }));
        } catch (e) {
             console.error(`Failed to load tables for ${namespaceParts.join('.')}:`, e);
             return [];
        }
    }

    function handleContextMenu(event: CustomEvent) {
        const { originalEvent, node } = event.detail;
        contextMenu = {
            visible: true,
            x: originalEvent.clientX,
            y: originalEvent.clientY,
            node
        };
    }

    function closeContextMenu() {
        contextMenu.visible = false;
    }

    function handleGlobalClick() {
        if (contextMenu.visible) closeContextMenu();
    }

    function openCreateNamespace() {
        showCreateNamespace = true;
        closeContextMenu();
    }

    function openCreateTable() {
        showCreateTable = true;
        closeContextMenu();
    }

    async function handleCreateNamespace(e: CustomEvent) {
        const { name } = e.detail;
        const catalogName = contextMenu.node.label; // For catalog node, label is name
        
        creating = true;
        try {
            await icebergApi.createNamespace(catalogName, { namespace: [name] });
            notifications.success(`Namespace '${name}' created successfully`);
            showCreateNamespace = false;
            triggerExplorerRefresh('catalog'); // Refresh catalog to show new namespace
        } catch (err: any) {
            console.error(err);
            notifications.error(`Failed to create namespace: ${err.message}`);
        } finally {
            creating = false;
        }
    }

    async function handleCreateTable(e: CustomEvent) {
        const request = e.detail;
        const node = contextMenu.node;
        
        // Context depends on node type:
        // If node is namespace, we have the path.
        // We need to parse catalog and namespace from href or node structure?
        // Node structure in ExplorerNode is generic.
        // Href usually: /explorer/{catalog}/{namespace...}
        // Let's rely on href to parse context.
        const pathParts = node.href.split('/').slice(2); // Remove empty and 'explorer'
        const catalogName = pathParts[0];
        const namespace = pathParts.slice(1);

        creating = true;
        try {
            await icebergApi.createTable(catalogName, namespace, request);
            notifications.success(`Table '${request.name}' created successfully`);
            showCreateTable = false;
            triggerExplorerRefresh('content'); 
        } catch (err: any) {
             console.error(err);
            notifications.error(`Failed to create table: ${err.message}`);
        } finally {
            creating = false;
        }
    }

    onMount(() => {
        loadCatalogs();
        document.addEventListener('click', handleGlobalClick);
        
        unsubscribe = refreshExplorer.subscribe(val => {
            if (val.timestamp > 0 && (val.type === 'catalog' || val.type === 'all')) {
                loadCatalogs();
            }
        });
    });

    onDestroy(() => {
        if (unsubscribe) unsubscribe();
        if (typeof document !== 'undefined') {
            document.removeEventListener('click', handleGlobalClick);
        }
    });
</script>

<div class="h-full flex flex-col relative">
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
                <ExplorerNode 
                    {...node} 
                    on:contextmenu={handleContextMenu}
                />
            {/each}
        {/if}
    </div>

    <!-- Context Menu -->
    {#if contextMenu.visible}
        <div 
            class="absolute bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 shadow-lg rounded-md py-1 z-50 text-sm min-w-[150px]"
            style="top: {contextMenu.y}px; left: {contextMenu.x}px; position: fixed;"
            on:click|stopPropagation
        >
            {#if contextMenu.node.type === 'catalog'}
                <button 
                    class="w-full text-left px-4 py-2 hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-700 dark:text-gray-200 flex items-center gap-2"
                    on:click={openCreateNamespace}
                >
                    <span class="material-icons text-xs">folder_open</span> New Namespace
                </button>
                <button 
                     class="w-full text-left px-4 py-2 hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-700 dark:text-gray-200 flex items-center gap-2"
                     on:click={() => triggerExplorerRefresh('content')}
                >
                    <span class="material-icons text-xs">refresh</span> Refresh
                </button>
            {:else if contextMenu.node.type === 'namespace'}
                <button 
                    class="w-full text-left px-4 py-2 hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-700 dark:text-gray-200 flex items-center gap-2"
                    on:click={openCreateTable}
                >
                    <span class="material-icons text-xs">table_chart</span> New Table
                </button>
                 <button 
                     class="w-full text-left px-4 py-2 hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-700 dark:text-gray-200 flex items-center gap-2"
                     on:click={() => triggerExplorerRefresh('content')}
                >
                    <span class="material-icons text-xs">refresh</span> Refresh
                </button>
            {/if}
        </div>
    {/if}

    <CreateNamespaceDialog 
        bind:open={showCreateNamespace} 
        loading={creating}
        on:create={handleCreateNamespace}
    />

    <CreateTableDialog
        bind:open={showCreateTable}
        loading={creating}
        on:create={handleCreateTable}
    />
</div>
