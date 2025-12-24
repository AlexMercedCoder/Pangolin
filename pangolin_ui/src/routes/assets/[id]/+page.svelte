<script lang="ts">
    import { onMount } from 'svelte';
    import { page } from '$app/stores';
    import { authStore } from '$lib/stores/auth';
    import TagInput from '$lib/components/ui/TagInput.svelte';
    
    $: token = $authStore.token;
    import { fade } from 'svelte/transition';

    let assetId = $page.params.id;
    let asset: any = null;
    let metadata: any = null;
    let catalogName = '';
    let namespaceName = '';
    let loading = true;
    let error = '';
    let activeTab = 'overview';

    // Metadata Edit State
    let isEditing = false;
    let editForm = {
        description: '',
        tags: [] as string[],
        discoverable: false,
        properties: '' // JSON string
    };

    async function fetchAsset() {
        loading = true;
        try {
            const res = await fetch(`/api/v1/assets/${assetId}`, {
                headers: { 'Authorization': `Bearer ${token}` }
            });
            if (res.ok) {
                const data = await res.json();
                asset = data.asset;
                metadata = data.metadata;
                catalogName = data.catalog;
                namespaceName = data.namespace;
                
                // Init edit form
                editForm.description = metadata?.description || '';
                editForm.tags = metadata?.tags || [];
                editForm.discoverable = metadata?.discoverable || false;
                editForm.properties = JSON.stringify(metadata?.properties || {}, null, 2);
            } else {
                error = 'Asset not found or access denied';
            }
        } catch (e: any) { error = e.message; }
        finally { loading = false; }
    }

    async function saveMetadata() {
        try {
            let props = {};
            try { props = JSON.parse(editForm.properties); } catch { alert('Invalid Properties JSON'); return; }

            const payload = {
                description: editForm.description,
                tags: editForm.tags,
                properties: props,
                discoverable: editForm.discoverable
            };

            const res = await fetch(`/api/v1/assets/${assetId}/metadata`, {
                method: 'POST',
                headers: { 
                    'Authorization': `Bearer ${token}`,
                    'Content-Type': 'application/json' 
                },
                body: JSON.stringify(payload)
            });

            if (res.ok) {
                isEditing = false;
                fetchAsset(); // Refresh
            } else {
                alert('Failed to save metadata');
            }
        } catch(e) { console.error(e); }
    }

    onMount(() => {
        fetchAsset();
    });
</script>

{#if loading}
    <div class="loading">Loading asset details...</div>
{:else if error}
    <div class="error-page">
        <span class="material-icons">error</span>
        <h1>Error</h1>
        <p>{error}</p>
        <a href="/search" class="primary-btn">Back to Search</a>
    </div>
{:else}
    <div class="asset-page">
        <!-- Header -->
        <div class="hero">
            <div class="breadcrumb">
                <span>{catalogName}</span> / <span>{namespaceName}</span> / <span>{asset.name}</span>
            </div>
            <div class="title-row">
                <span class="material-icons asset-icon">
                    {asset.kind === 'IcebergTable' ? 'table_chart' : 'description'}
                </span>
                <h1>{asset.name}</h1>
                <div class="badges">
                    <span class="badge kind">{asset.kind}</span>
                    {#if metadata?.discoverable}
                        <span class="badge discoverable">Discoverable</span>
                    {/if}
                </div>
            </div>
            <p class="uuid">ID: {asset.id}</p>
        </div>

        <!-- Tabs -->
        <div class="tabs">
            <button class="tab" class:active={activeTab === 'overview'} on:click={() => activeTab = 'overview'}>Overview</button>
            <button class="tab" class:active={activeTab === 'schema'} on:click={() => activeTab = 'schema'}>Schema</button>
            <button class="tab" class:active={activeTab === 'snapshots'} on:click={() => activeTab = 'snapshots'}>Snapshots</button>
            <button class="tab" class:active={activeTab === 'metadata'} on:click={() => activeTab = 'metadata'}>Metadata</button>
        </div>

        <!-- Content -->
        <div class="content">
            {#if activeTab === 'overview'}
                <div class="panel overview" transition:fade>
                    <section>
                        <h3>Description</h3>
                        {#if metadata?.description}
                            <div class="markdown-body">
                                {metadata.description}
                                <!-- Ideally render markdown here -->
                            </div>
                        {:else}
                            <p class="placeholder">No description provided.</p>
                        {/if}
                    </section>
                    
                    <section>
                        <h3>Tags</h3>
                        <div class="tags">
                            {#if metadata?.tags?.length}
                                {#each metadata.tags as tag}
                                    <span class="tag">#{tag}</span>
                                {/each}
                            {:else}
                                <p class="placeholder">No tags.</p>
                            {/if}
                        </div>
                    </section>

                    <section>
                        <h3>Properties</h3>
                        {#if asset.properties}
                            <table class="props-table">
                                {#each Object.entries(asset.properties) as [k, v]}
                                    <tr>
                                        <td><strong>{k}</strong></td>
                                        <td>{v}</td>
                                    </tr>
                                {/each}
                            </table>
                        {/if}
                    </section>
                </div>
            {:else if activeTab === 'schema'}
                <div class="panel" transition:fade>
                    <p>Schema visualization coming soon...</p>
                    <!-- We would parse asset.properties['metadata_location'] -> read file -> show schema -->
                </div>
            {:else if activeTab === 'snapshots'}
                <div class="panel" transition:fade>
                    <p>Snapshot history coming soon...</p>
                </div>
             {:else if activeTab === 'metadata'}
                <div class="panel" transition:fade>
                    <div class="header-actions">
                        <h3>Business Metadata</h3>
                        {#if !isEditing}
                            <button class="primary-btn small" on:click={() => isEditing = true}>Edit</button>
                        {/if}
                    </div>

                    {#if isEditing}
                        <div class="edit-form">
                            <div class="form-group">
                                <label>Description (Markdown)</label>
                                <textarea bind:value={editForm.description} rows="5"></textarea>
                            </div>
                            <div class="form-group">
                                <label>Tags</label>
                                <TagInput 
                                    bind:tags={editForm.tags} 
                                    placeholder="Type and press Enter to add tags..." 
                                />
                            </div>
                            <div class="form-group">
                                <label>
                                    <input type="checkbox" bind:checked={editForm.discoverable} />
                                    Discoverable via Search
                                </label>
                            </div>
                            <div class="form-group">
                                <label>Custom Properties (JSON)</label>
                                <textarea bind:value={editForm.properties} rows="5" class="code-font"></textarea>
                            </div>
                            <div class="actions">
                                <button class="text-btn" on:click={() => isEditing = false}>Cancel</button>
                                <button class="primary-btn" on:click={saveMetadata}>Save Changes</button>
                            </div>
                        </div>
                    {:else}
                         <!-- View Mode already shown in Overview, maybe show raw here? -->
                         <pre>{JSON.stringify(metadata, null, 2)}</pre>
                    {/if}
                </div>
            {/if}
        </div>
    </div>
{/if}

<style>
    .asset-page { max-width: 1200px; margin: 0 auto; }
    
    .hero { margin-bottom: 2rem; }
    .breadcrumb { font-size: 0.875rem; color: var(--md-sys-color-on-surface-variant); margin-bottom: 1rem; }
    .breadcrumb span { opacity: 0.8; }
    
    .title-row { display: flex; align-items: center; gap: 1rem; margin-bottom: 0.5rem; }
    .asset-icon { font-size: 2rem; color: var(--md-sys-color-primary); background: var(--md-sys-color-primary-container); padding: 12px; border-radius: 12px; }
    h1 { margin: 0; font-size: 2.25rem; }
    
    .badges { display: flex; gap: 0.5rem; }
    .badge { padding: 4px 8px; border-radius: 4px; font-size: 0.75rem; font-weight: bold; text-transform: uppercase; }
    .badge.kind { background: var(--md-sys-color-secondary-container); color: var(--md-sys-color-on-secondary-container); }
    .badge.discoverable { background: var(--md-sys-color-tertiary-container); color: var(--md-sys-color-on-tertiary-container); }
    
    .uuid { font-family: monospace; font-size: 0.875rem; color: var(--md-sys-color-on-surface-variant); margin: 0; }

    .tabs { display: flex; border-bottom: 1px solid var(--md-sys-color-outline-variant); margin-bottom: 2rem; gap: 2rem; }
    .tab { background: none; border: none; padding: 1rem 0; font-size: 1rem; color: var(--md-sys-color-on-surface-variant); cursor: pointer; border-bottom: 3px solid transparent; font-weight: 500; transition: all 0.2s; }
    .tab:hover { color: var(--md-sys-color-primary); }
    .tab.active { color: var(--md-sys-color-primary); border-bottom-color: var(--md-sys-color-primary); }

    .panel { background: var(--md-sys-color-surface); padding: 2rem; border-radius: 16px; border: 1px solid var(--md-sys-color-outline-variant); }
    
    section { margin-bottom: 2rem; }
    h3 { margin-top: 0; font-size: 1.25rem; color: var(--md-sys-color-primary); border-bottom: 1px solid var(--md-sys-color-outline-variant); padding-bottom: 0.5rem; }
    
    .tags { display: flex; gap: 0.5rem; flex-wrap: wrap; }
    .tag { background: var(--md-sys-color-surface-variant); padding: 4px 12px; border-radius: 100px; font-size: 0.875rem; }
    
    .props-table { width: 100%; border-collapse: collapse; }
    .props-table td { padding: 0.5rem; border-bottom: 1px solid rgba(0,0,0,0.05); }
    .props-table tr:last-child td { border-bottom: none; }
    
    .header-actions { display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem; }
    
    .form-group { margin-bottom: 1.5rem; }
    .form-group label { display: block; margin-bottom: 0.5rem; font-weight: 500; }
    .form-group input:not([type="checkbox"]), textarea { width: 100%; padding: 0.75rem; border-radius: 8px; border: 1px solid var(--md-sys-color-outline); background: var(--md-sys-color-surface-container); color: inherit; box-sizing: border-box; font-family: inherit; }
    .form-group input[type="checkbox"] { width: 1.25rem; height: 1.25rem; margin-right: 0.5rem; vertical-align: middle; accent-color: var(--md-sys-color-primary); }
    .code-font { font-family: monospace; }
    
    .actions { display: flex; justify-content: flex-end; gap: 1rem; }
    
    .primary-btn { background: var(--md-sys-color-primary); color: var(--md-sys-color-on-primary); border: none; padding: 0.75rem 1.5rem; border-radius: 100px; cursor: pointer; font-weight: 500; text-decoration: none; display: inline-block; }
    .primary-btn.small { padding: 0.5rem 1rem; font-size: 0.875rem; }
    .text-btn { background: none; border: none; color: var(--md-sys-color-primary); cursor: pointer; font-weight: 500; }
    
    .error-page { text-align: center; padding: 4rem; }
    .error-page .material-icons { font-size: 4rem; color: var(--md-sys-color-error); margin-bottom: 1rem; }
</style>
