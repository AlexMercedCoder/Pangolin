<script lang="ts">
    import { onMount } from 'svelte';

    let tenants: any[] = [];
    let error = "";

    onMount(async () => {
        try {
            const res = await fetch('http://localhost:8080/api/v1/tenants');
            if (res.ok) {
                tenants = await res.json();
            } else {
                error = "Failed to load tenants";
            }
        } catch (e) {
            error = e.message;
        }
    });
</script>

<div class="p-4">
    <h1>Tenants</h1>
    
    {#if error}
        <div class="error">
            {error}
        </div>
    {/if}

    <div class="grid">
        {#each tenants as tenant}
            <div class="card">
                <h2>{tenant.name}</h2>
                <p>ID: {tenant.id}</p>
            </div>
        {/each}
    </div>
</div>
