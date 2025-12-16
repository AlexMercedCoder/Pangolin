	import { triggerExplorerRefresh } from '$lib/stores/explorer';

    // ... imports

    // ... logic ...

</script>

<div class="h-full flex flex-col">
	<div class="p-4 border-b border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800 flex justify-between items-center">
		<h2 class="text-sm font-semibold uppercase tracking-wider text-gray-500 dark:text-gray-400">
			Data Explorer
		</h2>
        <button 
            class="p-1 hover:bg-gray-200 dark:hover:bg-gray-700 rounded text-gray-500 transition-colors"
            title="Refresh"
            on:click={() => triggerExplorerRefresh('all')}
        >
            🔄
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
			</div>
		{:else if nodes.length === 0}
			<div class="p-4 text-gray-500 text-sm text-center">
				No catalogs found.
			</div>
		{:else}
			{#each nodes as node (node.id)}
				<ExplorerNode {...node} />
			{/each}
		{/if}
	</div>
</div>
