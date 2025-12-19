<script lang="ts">
    import { helpStore } from '$lib/stores/help';
    import { fade, fly } from 'svelte/transition';

    function close() {
        helpStore.close();
    }
</script>

{#if $helpStore.isOpen}
    <div 
        class="fixed inset-0 z-50 flex justify-end"
        transition:fade={{ duration: 200 }}
    >
        <!-- Backdrop -->
        <div 
            class="absolute inset-0 bg-black/50 backdrop-blur-sm"
            on:click={close}
        ></div>

        <!-- Panel -->
        <div 
            class="relative w-full max-w-md bg-white dark:bg-gray-800 shadow-2xl flex flex-col h-full right-0"
            transition:fly={{ x: 500, duration: 300 }}
        >
            <!-- Header -->
            <div class="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
                <h2 class="text-xl font-semibold text-gray-900 dark:text-white">
                    Help & Resources
                </h2>
                <button 
                    on:click={close}
                    class="p-2 rounded-lg text-gray-500 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
                >
                    <span class="material-icons">close</span>
                </button>
            </div>

            <!-- Content -->
            <div class="flex-1 overflow-y-auto p-6">
                <div class="text-center space-y-6">
                    <div class="w-16 h-16 bg-primary-100 dark:bg-primary-900/30 rounded-full flex items-center justify-center mx-auto text-primary-600 dark:text-primary-400">
                        <span class="material-icons text-3xl">menu_book</span>
                    </div>
                    
                    <div>
                        <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-2">Documentation</h3>
                        <p class="text-gray-600 dark:text-gray-400 mb-6">
                            Visit our comprehensive documentation on GitHub to learn more about features and usage.
                        </p>
                        
                        <a 
                            href="https://github.com/AlexMercedCoder/Pangolin/tree/main/docs" 
                            target="_blank" 
                            class="inline-flex items-center justify-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-primary-600 hover:bg-primary-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500 w-full"
                        >
                            Open Documentation
                            <span class="material-icons text-sm ml-2">open_in_new</span>
                        </a>
                    </div>

                    <div class="border-t border-gray-200 dark:border-gray-700 pt-6">
                         <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-2">Website</h3>
                         <a 
                            href="https://pangolincatalog.org/" 
                            target="_blank" 
                            class="text-primary-600 hover:underline block"
                        >
                            pangolincatalog.org
                        </a>
                    </div>
                </div>
            </div>
        </div>
    </div>
{/if}

<style>
    :global(.prose) {
        font-size: 0.95rem;
        line-height: 1.6;
    }
    :global(.prose h1) { margin-top: 0; }
    :global(.prose code) {
        background-color: rgba(0,0,0,0.05);
        padding: 0.2rem 0.4rem;
        border-radius: 4px;
        font-size: 0.85em;
    }
    :global(dark .prose code) {
        background-color: rgba(255,255,255,0.1);
    }
</style>
