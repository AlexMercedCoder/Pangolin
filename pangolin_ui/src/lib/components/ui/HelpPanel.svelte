<script lang="ts">
    import { helpStore } from '$lib/stores/help';
    import { fade, fly } from 'svelte/transition';
    import { onMount } from 'svelte';
    import { marked } from 'marked';

    let content = '';
    let loading = false;
    let error = '';

    $: if ($helpStore.isOpen && $helpStore.docPath) {
        loadDoc($helpStore.docPath);
    }

    async function loadDoc(path: string) {
        loading = true;
        error = '';
        try {
            const res = await fetch(`/api/docs/${path}`);
            if (!res.ok) throw new Error('Failed to load documentation');
            const data = await res.json();
            content = await marked.parse(data.content);
        } catch (e: any) {
            error = e.message;
            content = '';
        } finally {
            loading = false;
        }
    }

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
            class="relative w-full max-w-2xl bg-white dark:bg-gray-800 shadow-2xl flex flex-col h-full"
            transition:fly={{ x: 500, duration: 300 }}
        >
            <!-- Header -->
            <div class="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
                <h2 class="text-xl font-semibold text-gray-900 dark:text-white">
                    {$helpStore.title}
                </h2>
                <button 
                    on:click={close}
                    class="p-2 rounded-lg text-gray-500 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
                >
                    <span class="material-icons">close</span>
                </button>
            </div>

            <!-- Content -->
            <div class="flex-1 overflow-y-auto p-6 custom-scrollbar">
                {#if loading}
                    <div class="flex flex-col items-center justify-center h-64 space-y-4">
                        <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600"></div>
                        <p class="text-gray-500">Loading documentation...</p>
                    </div>
                {:else if error}
                    <div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4 text-center">
                        <p class="text-red-600 dark:text-red-400 font-medium">{error}</p>
                        <button 
                            on:click={() => loadDoc($helpStore.docPath!)}
                            class="mt-4 text-sm font-semibold text-primary-600 hover:text-primary-700"
                        >
                            Try Again
                        </button>
                    </div>
                {:else}
                    <div class="prose prose-slate dark:prose-invert max-w-none prose-headings:font-bold prose-a:text-primary-600">
                        {@html content}
                    </div>
                {/if}
            </div>

            <!-- Footer -->
            <div class="p-4 border-t border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800/50">
                <div class="text-xs text-gray-500 text-center space-y-1">
                    <div>
                        Need more help? Check the <a href="https://github.com/AlexMercedCoder/Pangolin/tree/main/docs" target="_blank" class="text-primary-600 hover:underline">online documentation</a>.
                    </div>
                    <div>
                        Visit <a href="https://pangolincatalog.org/" target="_blank" class="text-primary-600 hover:underline">pangolincatalog.org</a> for more resources.
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
