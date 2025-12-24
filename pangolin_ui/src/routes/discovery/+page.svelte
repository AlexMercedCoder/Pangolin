<script lang="ts">
	import { onMount } from 'svelte';
	import { businessMetadataApi, type BusinessMetadata } from '$lib/api/business_metadata';
    import type { Asset } from '$lib/api/iceberg'; // Import Asset type
    import { notifications } from '$lib/stores/notifications';
    import { authStore } from '$lib/stores/auth'; // Need auth to know if we can edit
    import RequestAccessModal from '$lib/components/discovery/RequestAccessModal.svelte';

	let query = '';
	let results: any[] = []; // Asset + Metadata combined
	let loading = false;
    let requestReason = '';
    let selectedAssetId: string | null = null;
    let showRequestModal = false;
    
    // Edit Mode State
    let editingAssetId: string | null = null;
    let editForm: {
        description: string;
        tags: string;
        discoverable: boolean;
    } = { description: '', tags: '', discoverable: false };

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


    // Updated to use shared RequestAccessModal
    function openRequestModal(assetId: string) {
        selectedAssetId = assetId;
        showRequestModal = true;
    }


    function startEditing(asset: any) {
        editingAssetId = asset.id;
        editForm = {
            description: asset.description || '',
            tags: (asset.tags || []).join(', '),
            discoverable: asset.discoverable || false
        };
    }

    function cancelEditing() {
        editingAssetId = null;
    }

    async function saveMetadata(assetId: string) {
        try {
            await businessMetadataApi.addMetadata(assetId, {
                description: editForm.description,
                tags: editForm.tags.split(',').map(t => t.trim()).filter(t => t),
                discoverable: editForm.discoverable
            });
            notifications.success('Metadata updated');
            editingAssetId = null;
            handleSearch(); // Refresh results
        } catch (e) {
            console.error(e);
            notifications.error('Failed to update metadata');
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
            <div class="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6 flex flex-col relative group">
                <!-- Edit Button (Visible on Hover for authorized users) -->
                 <!-- Ideally check permission first, simpler for now: show if logged in -->
                 {#if $authStore.user && editingAssetId !== asset.id}
                    <button 
                        class="absolute top-2 right-2 p-1 text-gray-400 hover:text-primary-500 opacity-0 group-hover:opacity-100 transition-opacity"
                        on:click={() => startEditing(asset)}
                        title="Edit Metadata"
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                            <path d="M13.586 3.586a2 2 0 112.828 2.828l-.793.793-2.828-2.828.793-.793zM11.379 5.793L3 14.172V17h2.828l8.38-8.379-2.83-2.828z" />
                        </svg>
                    </button>
                 {/if}

                {#if editingAssetId === asset.id}
                    <!-- Edit Mode -->
                    <div class="space-y-4 flex-1">
                        <div>
                             <label class="block text-xs font-bold text-gray-500 uppercase mb-1">Description</label>
                             <textarea 
                                bind:value={editForm.description}
                                class="w-full text-sm p-2 border rounded bg-gray-50 dark:bg-gray-900 dark:border-gray-600"
                                rows="3"
                             ></textarea>
                        </div>
                        <div>
                            <label class="block text-xs font-bold text-gray-500 uppercase mb-1">Tags (comma separated)</label>
                            <input 
                               type="text" 
                               bind:value={editForm.tags}
                               class="w-full text-sm p-2 border rounded bg-gray-50 dark:bg-gray-900 dark:border-gray-600"
                            />
                        </div>
                        <div class="flex items-center gap-2">
                            <input type="checkbox" id="disc-{asset.id}" bind:checked={editForm.discoverable} />
                            <label for="disc-{asset.id}" class="text-sm select-none">Discoverable</label>
                        </div>
                        <div class="flex justify-end gap-2 mt-2">
                             <button on:click={cancelEditing} class="text-xs px-2 py-1 text-gray-500 hover:text-gray-700">Cancel</button>
                             <button on:click={() => saveMetadata(asset.id)} class="text-xs px-2 py-1 bg-primary-600 text-white rounded hover:bg-primary-700">Save</button>
                        </div>
                    </div>
                {:else}
                    <!-- View Mode -->
                    <div class="flex items-start justify-between mb-4">
                        <div>
                             <h3 class="text-lg font-semibold text-gray-900 dark:text-white">{asset.name}</h3>
                             <p class="text-sm text-gray-500 dark:text-gray-400">{asset.kind}</p>
                        </div>
                         <span class="px-2 py-1 text-xs font-medium rounded-full bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200">
                            {asset.kind}
                        </span>
                    </div>
                    
                    <p class="text-gray-600 dark:text-gray-300 mb-4 flex-1 text-sm">
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
                        <span class="text-xs text-gray-500 truncate max-w-[50%]">
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
                        {:else}
                             <span class="text-xs text-gray-400 italic">Not Discoverable</span>
                        {/if}
                    </div>
                {/if}
            </div>
        {/each}
        
        {#if results.length === 0 && query && !loading}
             <div class="col-span-full text-center py-12 text-gray-500">
                No assets found matching "{query}"
            </div>
        {/if}
	</div>
</div>

<RequestAccessModal 
    bind:open={showRequestModal} 
    assetId={selectedAssetId} 
    assetName={selectedAssetId ? results.find(r => r.id === selectedAssetId)?.name : 'Asset'} 
/>
