<script lang="ts">
  import { searchApi } from '$lib/api/optimization';
  import type { AssetSearchResult } from '$lib/types/optimization';
  import { fade } from 'svelte/transition';
  import { goto } from '$app/navigation';
  import { onMount, onDestroy } from 'svelte';

  import RequestAccessModal from '$lib/components/discovery/RequestAccessModal.svelte';

  let searchQuery = '';
  let results: AssetSearchResult[] = [];
  let showResults = false;
  let loading = false;
  let debounceTimer: any;
  let searchContainer: HTMLElement;
  
  // Modal state
  let showRequestModal = false;
  let selectedAsset: AssetSearchResult | null = null;

  function debounce(func: Function, wait: number) {
    return (...args: any[]) => {
      clearTimeout(debounceTimer);
      debounceTimer = setTimeout(() => func(...args), wait);
    };
  }

  const performSearch = debounce(async (query: string) => {
    if (!query || query.length < 2) {
      results = [];
      return;
    }
    
    loading = true;
    try {
      const response = await searchApi.searchAssets({ q: query, limit: 10 });
      results = response.results;
      if (results.length > 0) {
          showResults = true;
      }
    } catch (error) {
      console.error('Search failed', error);
      results = [];
    } finally {
      loading = false;
    }
  }, 300);

  $: performSearch(searchQuery);

  function handleKeydown(event: KeyboardEvent) {
      if (event.key === 'Escape') {
          showResults = false;
      }
  }

  function handleClickOutside(event: MouseEvent) {
      if (searchContainer && !searchContainer.contains(event.target as Node)) {
          showResults = false;
      }
  }

  function handleSelect(result: AssetSearchResult) {
      // If modal is open, don't navigate
      if (showRequestModal) return;

      console.log('Selecting search result:', result);
      showResults = false;
      searchQuery = '';
      
      // Navigate to catalog
      // Navigate based on asset type
        if (result.catalog) {
            if (result.asset_type === 'table' || result.asset_type === 'view') {
                // Navigate to explorer with asset ID for context (e.g. for request access)
                const namespacePath = result.namespace.join('/');
                goto(`/explorer/${encodeURIComponent(result.catalog)}/${namespacePath}/${encodeURIComponent(result.name)}?id=${result.id}`);
            } else {
                // Fallback to catalog
                goto(`/catalogs/${encodeURIComponent(result.catalog)}`);
            }
        } else {
          console.error('Result missing catalog field', result);
      }
  }
  
  function openRequestModal(e: MouseEvent, result: AssetSearchResult) {
      e.preventDefault();
      e.stopPropagation();
      selectedAsset = result;
      showRequestModal = true;
      showResults = false;
  }

  onMount(() => {
      document.addEventListener('click', handleClickOutside);
  });

  onDestroy(() => {
      if (typeof document !== 'undefined') {
        document.removeEventListener('click', handleClickOutside);
      }
  });
</script>

<div class="relative max-w-md w-full" bind:this={searchContainer}>
  <div class="relative">
    <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
      <span class="text-gray-400">🔍</span>
    </div>
    <input
      type="text"
      bind:value={searchQuery}
      on:focus={() => { if (results.length > 0) showResults = true; }}
      on:keydown={handleKeydown}
      class="block w-full pl-10 pr-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md leading-5 bg-gray-50 dark:bg-gray-700 placeholder-gray-500 focus:outline-none focus:bg-white dark:focus:bg-gray-600 focus:ring-1 focus:ring-primary-500 focus:border-primary-500 sm:text-sm text-gray-900 dark:text-white transition duration-150 ease-in-out"
      placeholder="Search assets..."
    />
    {#if loading}
      <div class="absolute inset-y-0 right-0 pr-3 flex items-center pointer-events-none">
        <svg class="animate-spin h-4 w-4 text-gray-400" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
        </svg>
      </div>
    {/if}
  </div>

  {#if showResults && results.length > 0}
    <div 
        transition:fade={{ duration: 100 }}
        class="absolute z-50 mt-1 w-full bg-white dark:bg-gray-800 shadow-lg max-h-96 rounded-md py-1 text-base ring-1 ring-black ring-opacity-5 overflow-auto focus:outline-none sm:text-sm"
    >
      {#each results as result}

                <div 
                    class="p-4 border-b border-gray-100 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700 cursor-pointer flex items-center justify-between"
                    on:click={() => handleSelect(result)}
                >
                    <div>
                        <div class="font-medium text-gray-900 dark:text-white flex items-center gap-2">
                            {result.name}
                            {#if !result.has_access}
                                <button
                                    on:click={(e) => openRequestModal(e, result)}
                                    class="text-xs px-1.5 py-0.5 rounded bg-gray-200 hover:bg-gray-300 dark:bg-gray-600 dark:hover:bg-gray-500 text-gray-600 dark:text-gray-300 flex items-center gap-1 transition-colors"
                                >
                                    <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3" viewBox="0 0 20 20" fill="currentColor">
                                        <path fill-rule="evenodd" d="M5 9V7a5 5 0 0110 0v2a2 2 0 012 2v5a2 2 0 01-2 2H5a2 2 0 01-2-2v-5a2 2 0 012-2zm8-2v2H7V7a3 3 0 016 0z" clip-rule="evenodd" />
                                    </svg>
                                    Request Access
                                </button>
                            {/if}
                        </div>
                        <div class="text-sm text-gray-500">
                           {result.catalog}.{result.namespace.join('.')}
                        </div>
                    </div>
                    <div class="text-xs px-2 py-1 rounded bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400">
                        {result.asset_type}
                    </div>
                </div>
      {/each}
      
      <div class="border-t border-gray-200 dark:border-gray-700 mt-1 pt-1 px-3 py-2 bg-gray-50 dark:bg-gray-900 text-xs text-center text-gray-500">
          Showing top {results.length} results
      </div>
    </div>
  {:else if showResults && results.length === 0 && !loading && searchQuery.length >= 2}
     <div 
        class="absolute z-50 mt-1 w-full bg-white dark:bg-gray-800 shadow-lg rounded-md py-4 text-center text-sm text-gray-500 dark:text-gray-400 ring-1 ring-black ring-opacity-5"
    >
        No results found
    </div>
  {/if}
</div>

<RequestAccessModal 
    bind:open={showRequestModal} 
    assetId={selectedAsset?.id || null} 
    assetName={selectedAsset?.name || 'Asset'} 
/>
