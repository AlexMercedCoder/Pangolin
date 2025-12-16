<script lang="ts">
    import { onMount } from 'svelte';

    let assets: any[] = [];
    let error = "";
    const tenantId = "00000000-0000-0000-0000-000000000000";
    const catalog = "default";
    const namespace = "default"; // Simplified for MVP

    onMount(async () => {
        try {
            // This endpoint might need adjustment based on actual API structure for listing all assets
            // For now, listing tables in default namespace
            const res = await fetch(`http://localhost:8080/v1/${catalog}/namespaces/${namespace}/tables`, {
                headers: {
                    'X-Pangolin-Tenant': tenantId
                }
            });
            if (res.ok) {
                const data = await res.json();
                assets = data.identifiers || [];
            } else {
                error = "Failed to load assets";
            }
        } catch (e: any) {
            error = e.message;
        }
    });
</script>

<div class="p-4">
    <h1>Assets (Default Namespace)</h1>
    
    {#if error}
        <div class="error">
            {error}
        </div>
    {/if}

    <div class="grid">
        {#each assets as asset}
            <div class="card">
                <h2>{asset.name}</h2>
                <p>Namespace: {asset.namespace.join('.')}</p>
            </div>
        {/each}
    </div>
</div>
