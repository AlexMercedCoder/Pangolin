<script lang="ts">
    import { onMount } from 'svelte';
    import { token } from '$lib/stores/auth';
    import { fade, scale } from 'svelte/transition';

    let permissions: any[] = []; // This might need to be fetched per user or all if admin?
    // API doesn't seem to have "List ALL permissions" endpoint in implementation plan
    // GET /api/v1/permissions/user/{id} exists.
    // If we want a global view, maybe we need to fetch all users and their permissions? Or just focus on "Grant" for now.
    
    // Let's implement a "User Permission Viewer" approach.
    // Select User -> See/Edit Permissions.
    
    let users: any[] = [];
    let selectedUser: any = null;
    let userPermissions: any[] = [];
    let loading = false;
    let showGrant = false;

    // Grant State
    let newPerm = {
        scopeType: 'Catalog',
        catalogId: '',
        namespace: '',
        assetId: '',
        tagName: '',
        actions: [] as string[]
    };

    const SCOPES = ['Catalog', 'Namespace', 'Asset', 'Tag'];
    const ACTIONS = ['read', 'write', 'delete', 'create', 'update', 'list', 'all', 'ingest-branching', 'experimental-branching'];

    async function fetchUsers() {
        const res = await fetch('/api/v1/users', { headers: { 'Authorization': `Bearer ${$token}` } });
        if (res.ok) users = await res.json();
    }

    async function fetchPermissions(userId: string) {
        loading = true;
        try {
            const res = await fetch(`/api/v1/permissions/user/${userId}`, { headers: { 'Authorization': `Bearer ${$token}` } });
            if (res.ok) userPermissions = await res.json();
        } finally { loading = false; }
    }

    function selectUser(u: any) {
        selectedUser = u;
        fetchPermissions(u.id);
    }

    async function handleGrant() {
        if (!selectedUser) return;
        
        let scope: any = { type: newPerm.scopeType };
        if (newPerm.scopeType === 'Catalog') scope.catalog_id = newPerm.catalogId;
        if (newPerm.scopeType === 'Namespace') { scope.catalog_id = newPerm.catalogId; scope.namespace = newPerm.namespace; }
        if (newPerm.scopeType === 'Asset') { scope.catalog_id = newPerm.catalogId; scope.namespace = newPerm.namespace; scope.asset_id = newPerm.assetId; }
        if (newPerm.scopeType === 'Tag') { scope.tag_name = newPerm.tagName; }

        const payload = {
            user_id: selectedUser.id,
            scope,
            actions: newPerm.actions
        };

        try {
            const res = await fetch('/api/v1/permissions', {
                method: 'POST',
                headers: { 
                    'Authorization': `Bearer ${$token}`,
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify(payload)
            });

            if (res.ok) {
                showGrant = false;
                fetchPermissions(selectedUser.id);
                // Reset form
                newPerm.actions = [];
            } else {
                alert('Failed to grant permission');
            }
        } catch(e) { console.error(e); }
    }

    async function revokePermission(permId: string) {
        if (!confirm('Revoke this permission?')) return;
        try {
            const res = await fetch(`/api/v1/permissions/${permId}`, {
                method: 'DELETE',
                headers: { 'Authorization': `Bearer ${$token}` }
            });
            if (res.ok) {
                fetchPermissions(selectedUser.id);
            }
        } catch(e) { console.error(e); }
    }

    function toggleAction(action: string) {
        if (newPerm.actions.includes(action)) {
            newPerm.actions = newPerm.actions.filter(a => a !== action);
        } else {
            newPerm.actions = [...newPerm.actions, action];
        }
    }

    onMount(() => {
        fetchUsers();
    });
</script>

<div class="page-container">
    <div class="sidebar">
        <h2>Users</h2>
        <div class="user-list">
            {#each users as u}
                <button class="user-item" class:selected={selectedUser?.id === u.id} on:click={() => selectUser(u)}>
                    <div class="avatar">{u.username.charAt(0).toUpperCase()}</div>
                    <span>{u.username}</span>
                </button>
            {/each}
        </div>
    </div>

    <div class="content">
        {#if selectedUser}
            <div class="header">
                <h2>Permissions: {selectedUser.username}</h2>
                <button class="primary-btn" on:click={() => showGrant = true}>
                    <span class="material-icons">add_moderator</span>
                    Grant Permission
                </button>
            </div>

            {#if loading}
                <div>Loading permissions...</div>
            {:else}
                <div class="perm-grid">
                    {#each userPermissions as perm}
                        <div class="perm-card">
                            <div class="perm-header">
                                <span class="scope-type">{perm.scope.type}</span>
                                <button class="icon-btn delete" on:click={() => revokePermission(perm.id)}>
                                    <span class="material-icons">close</span>
                                </button>
                            </div>
                            <div class="scope-details">
                                {#if perm.scope.catalog_id}<div><label>Catalog:</label> {perm.scope.catalog_id.slice(0,8)}...</div>{/if}
                                {#if perm.scope.namespace}<div><label>Namespace:</label> {perm.scope.namespace}</div>{/if}
                                {#if perm.scope.asset_id}<div><label>Asset:</label> {perm.scope.asset_id.slice(0,8)}...</div>{/if}
                                {#if perm.scope.tag_name}<div><label>Tag:</label> {perm.scope.tag_name}</div>{/if}
                            </div>
                            <div class="actions-list">
                                {#each perm.actions as action}
                                    <span class="action-tag">{action}</span>
                                {/each}
                            </div>
                        </div>
                    {/each}
                    {#if userPermissions.length === 0}
                        <div class="empty-state">
                            <span class="material-icons">lock_open</span>
                            <p>No explicit permissions found for this user.</p>
                        </div>
                    {/if}
                </div>
            {/if}
        {:else}
            <div class="empty-selection">
                <span class="material-icons">person_search</span>
                <p>Select a user to manage permissions</p>
            </div>
        {/if}
    </div>
</div>

<!-- Grant Dialog -->
{#if showGrant}
    <div class="modal-backdrop" transition:fade on:click={() => showGrant = false}>
        <div class="modal" on:click|stopPropagation transition:scale>
            <h2>Grant Permission</h2>
            
             <div class="builder">
                <div class="form-group">
                    <label>Scope Type</label>
                    <select bind:value={newPerm.scopeType}>
                        {#each SCOPES as scope}
                            <option value={scope}>{scope}</option>
                        {/each}
                    </select>
                </div>
                
                {#if newPerm.scopeType === 'Catalog' || newPerm.scopeType === 'Namespace' || newPerm.scopeType === 'Asset'}
                    <div class="form-group">
                        <label>Catalog ID</label>
                        <input bind:value={newPerm.catalogId} placeholder="UUID" />
                    </div>
                {/if}
                {#if newPerm.scopeType === 'Namespace' || newPerm.scopeType === 'Asset'}
                    <div class="form-group">
                        <label>Namespace</label>
                        <input bind:value={newPerm.namespace} placeholder="marketing.sales" />
                    </div>
                {/if}
                {#if newPerm.scopeType === 'Asset'}
                    <div class="form-group">
                        <label>Asset ID</label>
                        <input bind:value={newPerm.assetId} placeholder="UUID" />
                    </div>
                {/if}
                {#if newPerm.scopeType === 'Tag'}
                     <div class="form-group">
                        <label>Tag Name</label>
                        <input bind:value={newPerm.tagName} placeholder="pii" />
                    </div>
                {/if}
                
                <div class="form-group">
                    <label>Actions</label>
                    <div class="actions-select">
                        {#each ACTIONS as action}
                            <button 
                                class="chip {newPerm.actions.includes(action) ? 'active' : ''}" 
                                on:click={() => toggleAction(action)}
                            >
                                {action}
                            </button>
                        {/each}
                    </div>
                </div>
            </div>

            <div class="modal-actions">
                <button class="text-btn" on:click={() => showGrant = false}>Cancel</button>
                <button class="primary-btn" on:click={handleGrant} disabled={newPerm.actions.length === 0}>Grant</button>
            </div>
        </div>
    </div>
{/if}

<style>
    .page-container {
        display: grid;
        grid-template-columns: 250px 1fr;
        gap: 2rem;
        height: calc(100vh - 100px); /* Approx height minus header */
    }

    .sidebar {
        background-color: var(--md-sys-color-surface);
        border-radius: 16px;
        padding: 1rem;
        overflow-y: auto;
    }

    .user-list {
        display: flex; flex-direction: column; gap: 0.5rem;
    }

    .user-item {
        display: flex; align-items: center; gap: 0.75rem;
        padding: 0.75rem;
        background: none; border: none; cursor: pointer;
        border-radius: 8px;
        color: inherit;
        transition: background-color 0.2s;
        text-align: left;
    }

    .user-item:hover { background-color: rgba(0,0,0,0.05); }
    .user-item.selected { background-color: var(--md-sys-color-secondary-container); color: var(--md-sys-color-on-secondary-container); }

    .avatar {
        width: 24px; height: 24px; background: var(--md-sys-color-primary); color: #fff; border-radius: 50%;
        display: flex; justify-content: center; align-items: center; font-size: 0.75rem;
    }

    .content {
        background-color: var(--md-sys-color-surface);
        border-radius: 16px;
        padding: 2rem;
        overflow-y: auto;
    }

    .header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 2rem; }
    h2 { margin: 0; }

    .perm-grid {
        display: grid; grid-template-columns: repeat(auto-fill, minmax(300px, 1fr)); gap: 1rem;
    }

    .perm-card {
        background-color: var(--md-sys-color-surface-container);
        padding: 1rem; border-radius: 12px;
    }

    .perm-header {
        display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.5rem;
    }

    .scope-type { font-weight: bold; color: var(--md-sys-color-primary); }

    .scope-details { font-size: 0.875rem; opacity: 0.8; margin-bottom: 1rem; }
    .scope-details label { font-weight: 500; margin-right: 4px; }

    .actions-list { display: flex; flex-wrap: wrap; gap: 4px; }
    .action-tag { 
        font-size: 0.75rem; background: var(--md-sys-color-surface); padding: 2px 6px; border-radius: 4px; border: 1px solid var(--md-sys-color-outline);
    }

    .empty-state, .empty-selection {
        display: flex; flex-direction: column; align-items: center; justify-content: center; height: 300px;
        opacity: 0.5; text-align: center;
    }
    .empty-state .material-icons, .empty-selection .material-icons { font-size: 4rem; margin-bottom: 1rem; }

    /* Modal Styles (Reusable) */
    .modal-backdrop { position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0,0,0,0.5); display: flex; justify-content: center; align-items: center; z-index: 1000; }
    .modal { background: var(--md-sys-color-surface); padding: 2rem; border-radius: 28px; width: 100%; max-width: 500px; max-height: 90vh; overflow-y: auto; }
    
    .form-group { margin-bottom: 1rem; }
    .form-group label { display: block; margin-bottom: 0.5rem; font-size: 0.875rem; }
    .form-group input, select { width: 100%; padding: 0.75rem; border-radius: 4px; border: 1px solid var(--md-sys-color-outline); background: var(--md-sys-color-surface-container-highest); color: inherit; box-sizing: border-box; }

    .actions-select { display: flex; flex-wrap: wrap; gap: 0.5rem; }
    .chip {
        padding: 6px 12px; border-radius: 100px; border: 1px solid var(--md-sys-color-outline); background: none; color: inherit; cursor: pointer;
    }
    .chip.active { background: var(--md-sys-color-primary-container); color: var(--md-sys-color-on-primary-container); border-color: transparent; }

    .modal-actions { display: flex; justify-content: flex-end; gap: 1rem; margin-top: 2rem; }
    .primary-btn { background: var(--md-sys-color-primary); color: #fff; padding: 0.75rem 1.5rem; border: none; border-radius: 100px; cursor: pointer; display: flex; align-items: center; gap: 0.5rem; font-weight: 500; }
    .text-btn { background: none; border: none; color: var(--md-sys-color-primary); cursor: pointer; font-weight: 500; }
    .icon-btn { background: none; border: none; cursor: pointer; padding: 4px; border-radius: 50%; color: inherit; }
    .icon-btn.delete:hover { color: var(--md-sys-color-error); background: var(--md-sys-color-error-container); }
</style>
