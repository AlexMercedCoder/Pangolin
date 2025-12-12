<script lang="ts">
    import { onMount } from 'svelte';
    import { user, token } from '$lib/auth';
    import { goto } from '$app/navigation';

    let branches = [];
    let catalogs = [];
    let selectedCatalog = "";
    let newBranchName = "";
    let error = "";

    onMount(async () => {
        if (!$user) return;
        
        // Load catalogs
        const res = await fetch(`/api/v1/tenants/${$user.tenant_id}/catalogs`, {
            headers: { 'Authorization': `Bearer ${$token}` }
        });
        if (res.ok) {
            catalogs = await res.json();
            if (catalogs.length > 0) {
                selectedCatalog = catalogs[0].name;
                loadBranches();
            }
        }
    });

    async function loadBranches() {
        if (!selectedCatalog) return;
        const res = await fetch(`/api/v1/tenants/${$user.tenant_id}/catalogs/${selectedCatalog}/branches`, {
            headers: { 'Authorization': `Bearer ${$token}` }
        });
        if (res.ok) {
            branches = await res.json();
        }
    }

    async function createBranch() {
        if (!newBranchName) return;
        const res = await fetch(`/api/v1/tenants/${$user.tenant_id}/catalogs/${selectedCatalog}/branches`, {
            method: 'POST',
            headers: { 
                'Authorization': `Bearer ${$token}`,
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({ name: newBranchName })
        });
        
        if (res.ok) {
            newBranchName = "";
            loadBranches();
        } else {
            error = "Failed to create branch";
        }
    }
</script>

<div class="p-6">
    <h1 class="text-2xl font-bold mb-4">Branch Management</h1>

    <div class="mb-6">
        <label class="block text-sm font-medium mb-2">Select Catalog</label>
        <select bind:value={selectedCatalog} on:change={loadBranches} class="w-full p-2 border rounded bg-gray-800 text-white border-gray-700">
            {#each catalogs as catalog}
                <option value={catalog.name}>{catalog.name}</option>
            {/each}
        </select>
    </div>

    <div class="bg-gray-800 p-4 rounded mb-6">
        <h2 class="text-xl font-semibold mb-4">Create Branch</h2>
        <div class="flex gap-4">
            <input type="text" bind:value={newBranchName} placeholder="Branch Name" class="flex-1 p-2 rounded bg-gray-700 text-white border border-gray-600" />
            <button on:click={createBranch} class="bg-blue-600 hover:bg-blue-700 px-4 py-2 rounded">Create</button>
        </div>
        {#if error}
            <p class="text-red-500 mt-2">{error}</p>
        {/if}
    </div>

    <div class="bg-gray-800 p-4 rounded">
        <h2 class="text-xl font-semibold mb-4">Branches</h2>
        <div class="space-y-2">
            {#each branches as branch}
                <div class="flex justify-between items-center p-3 bg-gray-700 rounded">
                    <div>
                        <span class="font-medium">{branch.name}</span>
                        <span class="text-sm text-gray-400 ml-2">({branch.assets.length} assets)</span>
                    </div>
                    <div class="space-x-2">
                        <a href="/commits?branch={branch.name}&catalog={selectedCatalog}" class="text-blue-400 hover:text-blue-300">Commits</a>
                    </div>
                </div>
            {/each}
        </div>
    </div>
</div>
