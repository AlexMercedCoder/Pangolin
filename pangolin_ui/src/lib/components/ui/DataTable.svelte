<script lang="ts">
	import { createEventDispatcher } from 'svelte';

	export let columns: Array<{
		key: string;
		label: string;
		sortable?: boolean;
		width?: string;
	}> = [];
	// Props for server-side pagination
	export let serverSide = false;
	export let page = 1;
	export let pageSize = 10;
	export let hasNextPage = false;
	export let data: any[] = [];
	export let loading = false;
	export let emptyMessage = 'No data available';
	export let searchable = true;
	export let searchPlaceholder = 'Search...';

	const dispatch = createEventDispatcher();

	let searchQuery = '';
	let sortKey = '';
	let sortDirection: 'asc' | 'desc' = 'asc';

	// Client-side filtering/sorting (only used if !serverSide)
	$: filteredData = !serverSide && searchQuery
		? data.filter((row) =>
				columns.some((col) => {
					const value = row[col.key];
					return value?.toString().toLowerCase().includes(searchQuery.toLowerCase());
				})
		  )
		: data;

	$: sortedData = !serverSide && sortKey
		? [...filteredData].sort((a, b) => {
				const aVal = a[sortKey];
				const bVal = b[sortKey];
				const modifier = sortDirection === 'asc' ? 1 : -1;

				if (aVal < bVal) return -1 * modifier;
				if (aVal > bVal) return 1 * modifier;
				return 0;
		  })
		: filteredData;
	
	// Server-side data uses 'data' directly (assumed to be the page slice)
	$: displayData = serverSide ? data : sortedData;

	function handleSort(key: string) {
		if (serverSide) return; // Disable client-side sort for server-side data for now (or implement server sort later)
		if (sortKey === key) {
			sortDirection = sortDirection === 'asc' ? 'desc' : 'asc';
		} else {
			sortKey = key;
			sortDirection = 'asc';
		}
	}

	function handleRowClick(row: any) {
		dispatch('rowClick', row);
	}

	function handlePageChange(newPage: number) {
		dispatch('pageChange', newPage);
	}
</script>

<div class="space-y-4">
	{#if searchable && !serverSide}
		<div class="flex items-center gap-4">
			<div class="flex-1">
				<input
					type="text"
					bind:value={searchQuery}
					placeholder={searchPlaceholder}
					class="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:ring-2 focus:ring-primary-500 focus:border-transparent"
				/>
			</div>
			<slot name="actions" />
		</div>
	{/if}

	<div class="overflow-x-auto rounded-lg border border-gray-200 dark:border-gray-700">
		<table class="w-full">
			<thead class="bg-gray-50 dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700">
				<tr>
					{#each columns as column}
						<th
							class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider"
							style={column.width ? `width: ${column.width}` : ''}
						>
							{#if column.sortable && !serverSide}
								<button
									on:click={() => handleSort(column.key)}
									class="flex items-center gap-2 hover:text-gray-700 dark:hover:text-gray-200 transition-colors"
								>
									{column.label}
									{#if sortKey === column.key}
										<span class="text-primary-600">
											{sortDirection === 'asc' ? '↑' : '↓'}
										</span>
									{/if}
								</button>
							{:else}
								{column.label}
							{/if}
						</th>
					{/each}
				</tr>
			</thead>
			<tbody class="bg-white dark:bg-gray-900 divide-y divide-gray-200 dark:divide-gray-700">
				{#if loading}
					<tr>
						<td colspan={columns.length} class="px-6 py-12 text-center">
							<div class="flex items-center justify-center gap-3">
								<div
									class="w-6 h-6 border-3 border-primary-600 border-t-transparent rounded-full animate-spin"
								/>
								<span class="text-gray-600 dark:text-gray-400">Loading...</span>
							</div>
						</td>
					</tr>
				{:else if displayData.length === 0}
					<tr>
						<td colspan={columns.length} class="px-6 py-12 text-center">
							<div class="text-gray-500 dark:text-gray-400">
								<span class="text-4xl mb-2 block">📭</span>
								{emptyMessage}
							</div>
						</td>
					</tr>
				{:else}
					{#each displayData as row}
						<tr
							on:click={() => handleRowClick(row)}
							class="hover:bg-gray-50 dark:hover:bg-gray-800 cursor-pointer transition-colors"
						>
							{#each columns as column}
								<td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900 dark:text-white">
									<slot name="cell" {row} {column}>
										{row[column.key] ?? '-'}
									</slot>
								</td>
							{/each}
						</tr>
					{/each}
				{/if}
			</tbody>
		</table>
	</div>

	{#if !loading && (serverSide || displayData.length > 0)}
        <div class="flex items-center justify-between text-sm text-gray-600 dark:text-gray-400">
            {#if serverSide}
                <div>
                    Page {page}
                </div>
                <div class="flex gap-2">
                    <button
                        class="px-3 py-1 border border-gray-300 rounded hover:bg-gray-100 disabled:opacity-50"
                        disabled={page === 1}
                        on:click={() => handlePageChange(page - 1)}
                    >
                        Previous
                    </button>
                    <button
                        class="px-3 py-1 border border-gray-300 rounded hover:bg-gray-100 disabled:opacity-50"
                        disabled={!hasNextPage}
                        on:click={() => handlePageChange(page + 1)}
                    >
                        Next
                    </button>
                </div>
            {:else}
                <div>
                    Showing {displayData.length} of {data.length} items
                </div>
            {/if}
		</div>
	{/if}
</div>
