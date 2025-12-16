<script lang="ts">
    import { createEventDispatcher } from 'svelte';
    import { goto } from '$app/navigation';
    import { page } from '$app/stores';

    import { refreshExplorer } from '$lib/stores/explorer';
    import { onMount, onDestroy } from 'svelte';

    export let label: string;
    export let type: 'catalog' | 'namespace' | 'table';
    export let icon: string = '📁';
    export let expanded: boolean = false;
    export let hasChildren: boolean = true;
    export let loading: boolean = false;
    export let children: any[] = [];
    export let href: string = '';
    
    // Function to load children when expanded
    export let loadChildren: () => Promise<any[]> = async () => [];

    const dispatch = createEventDispatcher();
    let unsubscribe: () => void;

    $: isActive = $page.url.pathname === href;

    onMount(() => {
        unsubscribe = refreshExplorer.subscribe(async (event) => {
            // If expanded and not initial load, refresh children if content changed
            // 'content' or 'all' updates should trigger child reload
            if (expanded && event.timestamp > 0 && (event.type === 'content' || event.type === 'all')) {
                reloadChildren();
            }
        });
    });

    onDestroy(() => {
        if (unsubscribe) unsubscribe();
    });

    async function reloadChildren() {
        loading = true;
        try {
            children = await loadChildren();
        } catch (error) {
            console.error('Error reloading children:', error);
        } finally {
            loading = false;
        }
    }

    async function toggleExpand() {
        if (!hasChildren) return;
        
        expanded = !expanded;
        if (expanded && children.length === 0) {
            await reloadChildren();
        }
    }

    function handleClick() {
        if (href) {
            goto(href);
        }
    }
</script>

<div class="select-none">
    <div 
        class="flex items-center py-1 px-2 rounded-lg cursor-pointer transition-colors duration-150
        {isActive ? 'bg-primary-100 text-primary-900 dark:bg-primary-900/30 dark:text-primary-100' : 'hover:bg-gray-100 dark:hover:bg-gray-800 text-gray-700 dark:text-gray-300'}"
        on:click={handleClick}
        on:contextmenu|preventDefault={(e) => dispatch('contextmenu', { originalEvent: e, node: { label, type, href, icon } })}
        style="padding-left: 0.5rem"
    >
        {#if hasChildren}
            <button 
                class="mr-1 p-0.5 rounded hover:bg-gray-200 dark:hover:bg-gray-700 text-gray-400 focus:outline-none"
                on:click|stopPropagation={toggleExpand}
            >
                <span class="transform transition-transform duration-200 inline-block {expanded ? 'rotate-90' : ''}" style="font-size: 0.7rem;">
                    ▶
                </span>
            </button>
        {:else}
            <span class="w-4 mr-1"></span>
        {/if}

        <span class="mr-2 text-lg opacity-80">{icon}</span>
        <span class="truncate text-sm font-medium">{label}</span>
        
        {#if loading}
            <div class="ml-auto animate-spin h-3 w-3 border-2 border-gray-400 border-t-transparent rounded-full"></div>
        {/if}
    </div>

    {#if expanded}
        <div class="ml-2 border-l border-gray-200 dark:border-gray-700 pl-1">
            {#each children as child (child.id || child.label)}
                <svelte:self
                    {...child}
                />
            {/each}
            {#if children.length === 0 && !loading}
                <div class="pl-6 py-1 text-xs text-gray-400 italic">No items</div>
            {/if}
        </div>
    {/if}
</div>
