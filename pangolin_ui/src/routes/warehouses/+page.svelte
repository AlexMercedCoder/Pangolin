<script lang="ts">
    import { onMount } from 'svelte';

    let warehouses: any[] = [];
    let error = "";
    // Hardcoded tenant for MVP UI
    const tenantId = "00000000-0000-0000-0000-000000000000"; 

    onMount(async () => {
        try {
            const res = await fetch('http://localhost:8080/api/v1/warehouses', {
                headers: {
                    'X-Pangolin-Tenant': tenantId
                }
            });
            if (res.ok) {
                warehouses = await res.json();
            } else {
                error = "Failed to load warehouses";
            }
        } catch (e) {
            error = e.message;
        }
    });
</script>

<div class="p-4">
    <h1>Warehouses</h1>
    
    {#if error}
        <div class="error">
            {error}
        </div>
    {/if}

    <div class="grid">
        {#each warehouses as warehouse}
            <div class="card">
                <h2>{warehouse.name}</h2>
                <p>Type: {warehouse.storage_config.type}</p>
            </div>
        {/each}
    </div>
</div>
