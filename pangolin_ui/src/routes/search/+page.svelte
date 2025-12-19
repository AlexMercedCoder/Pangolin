<script lang="ts">
    import { onMount } from 'svelte';
    import { token } from '$lib/stores/auth';
    import { fade } from 'svelte/transition';

    let query = '';
    let results: any[] = [];
    let loading = false;
    let error = '';
    let hasSearched = false;

    // Filters
    let tags = ''; // Comma separated

    async function handleSearch() {
        if (!query) return;
        loading = true;
        error = '';
        hasSearched = true;
        
        try {
            const params = new URLSearchParams({ query });
            if (tags) {
                // Split tags
                const tagList = tags.split(',').map(t => t.trim()).filter(Boolean);
                tagList.forEach(t => params.append('tags', t));
            }

            const res = await fetch(`/api/v1/assets/search?${params}`, {
                headers: { 'Authorization': `Bearer ${$token}` }
            });

            if (res.ok) {
                const data = await res.json();
                results = data.results;
            } else if (res.status === 501) {
                error = 'Search functionality is not yet implemented on the backend.';
            } else {
                error = 'Search failed';
            }
        } catch (e: any) {
            error = e.message || String(e);
        } finally {
            loading = false;
        }
    }

    function handleKeydown(e: KeyboardEvent) {
        if (e.key === 'Enter') handleSearch();
    }
</script>

<div class="search-page">
    <div class="search-hero">
        <h1>Discover Data</h1>
        <p>Find tables, views, and dashboards across your organization</p>
        
        <div class="search-bar">
            <span class="material-icons search-icon">search</span>
            <input 
                type="text" 
                bind:value={query} 
                on:keydown={handleKeydown}
                placeholder="Search for assets (e.g. 'sales customers')..." 
            />
            <button class="primary-btn search-btn" on:click={handleSearch} disabled={loading}>
                {#if loading}Searching...{:else}Search{/if}
            </button>
        </div>

        <div class="filters">
            <!-- Simple tag filter for now -->
            <input class="filter-input" bind:value={tags} placeholder="Filter by tags (comma separated)..." />
        </div>
    </div>

    <div class="results-container">
        {#if error}
            <div class="error-state" transition:fade>
                <span class="material-icons">error_outline</span>
                <p>{error}</p>
            </div>
        {:else if loading}
            <div class="loading-state">
                <div class="spinner"></div>
            </div>
        {:else if hasSearched && results.length === 0}
            <div class="empty-state" transition:fade>
                <span class="material-icons">travel_explore</span>
                <p>No results found for "{query}"</p>
            </div>
        {:else if results.length > 0}
            <div class="grid">
                {#each results as result}
                    <div class="card asset-card">
                        <div class="card-header">
                            <span class="material-icons asset-icon">
                                {result.kind === 'IcebergTable' ? 'table_chart' : 'description'}
                            </span>
                            <div>
                                <h3><a href="/assets/{result.id}">{result.name}</a></h3>
                                <small>{result.namespace}</small>
                            </div>
                            <div class="discoverable-status">
                                {#if !result.has_access && result.discoverable}
                                    <span class="material-icons lock" title="Request Access">lock</span>
                                {/if}
                            </div>
                        </div>
                        <p class="desc">{result.description || 'No description provided.'}</p>
                        <div class="tags">
                            {#each result.tags as tag}
                                <span class="tag">#{tag}</span>
                            {/each}
                        </div>
                    </div>
                {/each}
            </div>
        {/if}
    </div>
</div>

<style>
    .search-page {
        max-width: 900px;
        margin: 0 auto;
    }

    .search-hero {
        text-align: center;
        margin-bottom: 3rem;
        padding: 3rem 1rem;
        background: linear-gradient(180deg, var(--md-sys-color-surface-container-low) 0%, var(--md-sys-color-background) 100%);
        border-radius: 28px;
    }

    h1 {
        font-size: 2.5rem;
        margin: 0 0 0.5rem;
        background: linear-gradient(45deg, var(--md-sys-color-primary), var(--md-sys-color-secondary));
        -webkit-background-clip: text;
        -webkit-text-fill-color: transparent;
    }

    .search-hero p { margin: 0 0 2rem; opacity: 0.7; font-size: 1.1rem; }

    .search-bar {
        display: flex;
        align-items: center;
        background: var(--md-sys-color-surface);
        border: 1px solid var(--md-sys-color-outline);
        border-radius: 100px; /* Pillow */
        padding: 0.5rem 0.5rem 0.5rem 1.5rem;
        box-shadow: 0 4px 12px rgba(0,0,0,0.05);
        max-width: 600px;
        margin: 0 auto;
        transition: box-shadow 0.2s, border-color 0.2s;
    }

    .search-bar:focus-within {
        box-shadow: 0 8px 16px rgba(0,0,0,0.1);
        border-color: var(--md-sys-color-primary);
    }

    .search-icon { color: var(--md-sys-color-on-surface-variant); margin-right: 0.5rem; }

    .search-bar input {
        flex: 1;
        border: none;
        outline: none;
        font-size: 1rem;
        background: transparent;
        color: var(--md-sys-color-on-surface);
    }

    .search-btn {
        padding: 0.75rem 1.5rem;
        border-radius: 100px;
        border: none;
        background: var(--md-sys-color-primary);
        color: var(--md-sys-color-on-primary);
        font-weight: 500;
        cursor: pointer;
        transition: opacity 0.2s;
    }

    .search-btn:hover { opacity: 0.9; }

    .filters { margin-top: 1rem; }
    .filter-input {
        background: transparent; border: none; border-bottom: 1px solid var(--md-sys-color-outline-variant);
        text-align: center; color: var(--md-sys-color-on-surface-variant); font-size: 0.875rem; width: 300px;
    }

    .error-state, .empty-state {
        text-align: center; padding: 2rem; opacity: 0.8;
    }
    .error-state .material-icons { font-size: 2rem; color: var(--md-sys-color-error); }
    .empty-state .material-icons { font-size: 2rem; color: var(--md-sys-color-outline); }

    .grid {
        display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: 1.5rem;
    }

    .asset-card {
        background: var(--md-sys-color-surface);
        border-radius: 16px;
        padding: 1.5rem;
        border: 1px solid var(--md-sys-color-outline-variant);
        transition: transform 0.2s, box-shadow 0.2s;
    }

    .asset-card:hover {
        transform: translateY(-2px);
        box-shadow: 0 4px 8px rgba(0,0,0,0.1);
    }

    .card-header { display: flex; align-items: flex-start; gap: 1rem; margin-bottom: 0.5rem; }
    .asset-icon { color: var(--md-sys-color-primary); background: var(--md-sys-color-primary-container); padding: 8px; border-radius: 8px; }
    .card-header h3 { margin: 0; font-size: 1.1rem; }
    .card-header h3 a { color: inherit; text-decoration: none; }
    .card-header small { color: var(--md-sys-color-on-surface-variant); }
    
    .desc { font-size: 0.875rem; color: var(--md-sys-color-on-surface-variant); display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden; margin-bottom: 1rem; }
    
    .tags { display: flex; flex-wrap: wrap; gap: 0.5rem; }
    .tag { background: var(--md-sys-color-surface-variant); padding: 2px 8px; border-radius: 4px; font-size: 0.75rem; color: var(--md-sys-color-on-surface-variant); }
    
    .loading-state { display: flex; justify-content: center; padding: 2rem; }
    .spinner { width: 24px; height: 24px; border: 2px solid var(--md-sys-color-primary); border-top-color: transparent; border-radius: 50%; animation: spin 1s linear infinite; }
    @keyframes spin { 0% { transform: rotate(0deg); } 100% { transform: rotate(360deg); } }
</style>
