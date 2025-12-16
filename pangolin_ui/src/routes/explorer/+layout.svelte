<script lang="ts">
	import ExplorerTree from '$lib/components/explorer/ExplorerTree.svelte';
</script>

<div class="flex h-[calc(100vh-8rem)] gap-6">
	<!-- Left Sidebar: Tree -->
	<div class="w-80 flex-shrink-0 bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700 overflow-hidden">
		<ExplorerTree />
	</div>

	<!-- Main Content: Slot -->
	<div class="flex-1 bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700 overflow-hidden flex flex-col">
		<slot />
	</div>
</div>
