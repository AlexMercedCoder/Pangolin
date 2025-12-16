<script lang="ts">
    import { onMount } from 'svelte';
    import { user, token, selectedTenant } from '$lib/auth';
    import { fade } from 'svelte/transition';

    let tenants: any[] = [];
    let isOpen = false;
    let menuRef: HTMLDivElement;
    let loading = false;

    async function fetchTenants() {
        if (!$token) return;
        loading = true;
        try {
            // Only Root user should really need this, and API should allow Root to list all.
            // If the endpoint /api/v1/tenants returns all for Root, we are good.
            const res = await fetch('/api/v1/tenants', {
                headers: {
                    'Authorization': `Bearer ${$token}`
                }
            });
            if (res.ok) {
                tenants = await res.json();
            }
        } catch (e) {
            console.error("Failed to fetch tenants", e);
        } finally {
            loading = false;
        }
    }

    function selectTenant(tenant: any) {
        selectedTenant.set(tenant.id);
        isOpen = false;
        // Optionally reload page or trigger data refresh
        window.location.reload(); 
    }

    function toggleMenu() {
        if (!isOpen && tenants.length === 0) {
            fetchTenants();
        }
        isOpen = !isOpen;
    }

    function handleClickOutside(event: MouseEvent) {
        if (menuRef && !menuRef.contains(event.target as Node)) {
            isOpen = false;
        }
    }

    onMount(() => {
        document.addEventListener('click', handleClickOutside);
        return () => {
            document.removeEventListener('click', handleClickOutside);
        };
    });

    // Reactive label
    $: currentTenantName = tenants.find(t => t.id === $selectedTenant)?.name || 'All Tenants';
</script>

{#if $user?.role === 'Root'}
<div class="tenant-selector" bind:this={menuRef}>
    <button class="selector-btn" on:click|stopPropagation={toggleMenu}>
        <span class="material-icons">business</span>
        <span class="label">{currentTenantName}</span>
        <span class="material-icons arrow">expand_more</span>
    </button>

    {#if isOpen}
        <div class="dropdown" transition:fade={{ duration: 100 }}>
            {#if loading}
                <div class="item loading">Loading...</div>
            {:else}
                <button class="item" on:click={() => selectTenant({id: null, name: 'All Tenants'})} class:selected={!$selectedTenant}>
                    All Tenants
                </button>
                {#each tenants as tenant}
                    <button class="item" on:click={() => selectTenant(tenant)} class:selected={$selectedTenant === tenant.id}>
                        {tenant.name}
                    </button>
                {/each}
            {/if}
        </div>
    {/if}
</div>
{/if}

<style>
    .tenant-selector {
        position: relative;
    }

    .selector-btn {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        background-color: var(--md-sys-color-surface-variant, #e0e0e0);
        color: var(--md-sys-color-on-surface-variant, #000);
        border: none;
        padding: 0.5rem 1rem;
        border-radius: 8px;
        cursor: pointer;
        font-size: 0.875rem;
        font-weight: 500;
        transition: background-color 0.2s;
    }

    .selector-btn:hover {
        background-color: var(--md-sys-color-inverse-on-surface); /* Darker */
    }

    .dropdown {
        position: absolute;
        top: 100%;
        left: 0;
        margin-top: 0.5rem;
        background-color: var(--md-sys-color-surface);
        color: var(--md-sys-color-on-surface);
        border-radius: 8px;
        box-shadow: 0 4px 6px -1px rgba(0,0,0,0.1);
        min-width: 200px;
        z-index: 50;
        max-height: 300px;
        overflow-y: auto;
        border: 1px solid rgba(0,0,0,0.05);
    }

    .item {
        display: block;
        width: 100%;
        text-align: left;
        padding: 0.75rem 1rem;
        background: none;
        border: none;
        color: inherit;
        cursor: pointer;
        font-size: 0.875rem;
    }

    .item:hover {
        background-color: rgba(0,0,0,0.05);
    }

    .item.selected {
        background-color: var(--md-sys-color-secondary-container);
        color: var(--md-sys-color-on-secondary-container);
    }

    .material-icons {
        font-size: 1.25rem;
    }
</style>
