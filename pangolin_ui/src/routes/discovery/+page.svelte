<script lang="ts">
	import { onMount } from 'svelte';
	import { businessMetadataApi, type BusinessMetadata } from '$lib/api/business_metadata';
    import type { Asset } from '$lib/api/iceberg'; // Import Asset type
    import { notifications } from '$lib/stores/notifications';

	let query = '';
	let results: any[] = []; // Asset + Metadata combined
	let loading = false;
    let requestReason = '';
    let selectedAssetId: string | null = null;
    let showRequestModal = false;

	async function handleSearch() {
        if (!query.trim()) return;
		loading = true;
		try {
            results = await businessMetadataApi.searchAssets(query);
		} catch (e) {
			console.error(e);
            notifications.error('Search failed');
		} finally {
			loading = false;
		}
	}

    function openRequestModal(assetId: string) {
        selectedAssetId = assetId;
        requestReason = '';
        showRequestModal = true;
    }

    async function submitRequest() {
        if (!selectedAssetId) return;
        try {
            await businessMetadataApi.requestAccess(selectedAssetId, { reason: requestReason });
            notifications.success('Access request submitted');
            showRequestModal = false;
        } catch (e) {
            console.error(e);
            notifications.error('Failed to submit request');
        }
    }
</script>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<h1 class="text-2xl font-bold text-gray-900 dark:text-white">Discovery Portal</h1>
	</div>

	<!-- Search Bar -->
	<div class="bg-white dark:bg-gray-800 p-6 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700">
		<div class="flex gap-4">
			<input
				type="text"
				bind:value={query}
                on:keydown={(e) => e.key === 'Enter' && handleSearch()}
				placeholder="Search for datasets (use # for tags, e.g. #public)..."
				class="flex-1 px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-gray-50 dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-primary-500 focus:border-transparent"
			/>
			<button
				on:click={handleSearch}
				disabled={loading}
				class="px-6 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50 transition-colors"
			>
				{loading ? 'Searching...' : 'Search'}
			</button>
		</div>
	</div>

	<!-- Results -->
	<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {#each results as asset}
            <div class="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6 flex flex-col">
                <div class="flex items-start justify-between mb-4">
                    <div>
                         <h3 class="text-lg font-semibold text-gray-900 dark:text-white">{asset.name}</h3>
                         <p class="text-sm text-gray-500 dark:text-gray-400">{asset.kind}</p>
                    </div>
                     <span class="px-2 py-1 text-xs font-medium rounded-full bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200">
                        {asset.kind}
                    </span>
                </div>
                
                <p class="text-gray-600 dark:text-gray-300 mb-4 flex-1">
                    {asset.description || 'No description provided.'}
                </p>
                
                {#if asset.tags && asset.tags.length > 0}
                <div class="flex flex-wrap gap-2 mb-6">
                    {#each asset.tags as tag}
                        <span class="px-2 py-1 text-xs rounded bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300">
                            #{tag}
                        </span>
                    {/each}
                </div>
                {/if}

                <div class="mt-auto pt-4 border-t border-gray-100 dark:border-gray-700 flex justify-between items-center">
                    <span class="text-xs text-gray-500">
                        {asset.catalog ? `${asset.catalog}.${asset.namespace}` : asset.location}
                    </span>
                    {#if !asset.has_access && asset.discoverable}
                        <button 
                            on:click={() => openRequestModal(asset.id)}
                            class="text-sm font-medium text-primary-600 hover:text-primary-700 dark:text-primary-400"
                        >
                            Request Access
                        </button>
                    {:else if asset.has_access}
                        <a 
                            href="/assets/{asset.id}"
                            class="text-sm font-medium text-primary-600 hover:text-primary-700 dark:text-primary-400"
                        >
                            View Asset
                        </a>
                    {/if}
                </div>
            </div>
        {/each}
        
        {#if results.length === 0 && query && !loading}
             <div class="col-span-full text-center py-12 text-gray-500">
                No assets found matching "{query}"
            </div>
        {/if}
	</div>
</div>

<!-- Request Model -->
{#if showRequestModal}
<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
    <div class="bg-white dark:bg-gray-800 rounded-lg shadow-xl w-full max-w-lg p-6">
        <h3 class="text-xl font-bold text-gray-900 dark:text-white mb-4">Request Access</h3>
        
        <div class="mb-4">
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                Reason for Access
            </label>
            <textarea
                bind:value={requestReason}
                rows="4"
                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-gray-50 dark:bg-gray-700 text-gray-900 dark:text-white"
                placeholder="Please describe why you need access to this dataset..."
            ></textarea>
        </div>
        
        <div class="flex justify-end gap-3">
            <button
                on:click={() => showRequestModal = false}
                class="px-4 py-2 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg"
            >
                Cancel
            </button>
            <button
                on:click={submitRequest}
                class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700"
            >
                Submit Request
            </button>
        </div>
    </div>
</div>
{/if}
