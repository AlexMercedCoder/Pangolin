<script lang="ts">
    import { onMount } from 'svelte';
    import { token } from '$lib/auth';
    import { fade, scale } from 'svelte/transition';

    let roles: any[] = [];
    let loading = true;
    let error = '';

    let showCreate = false;
    let showEdit = false;
    let showDelete = false;
    let selectedRole: any = null;

    // Permission Builder State
    let formRole = {
        name: '',
        permissions: [] as any[]
    };
    
    // Temp permission being added
    let newPerm = {
        scopeType: 'Catalog',
        catalogId: '',
        namespace: '',
        assetId: '',
        tagName: '',
        actions: [] as string[]
    };

    const SCOPES = ['Catalog', 'Namespace', 'Asset', 'Tag'];
    const ACTIONS = ['read', 'write', 'delete', 'create', 'update', 'list', 'all'];

    async function fetchRoles() {
        loading = true;
        try {
            const res = await fetch('/api/v1/roles', {
                headers: { 'Authorization': `Bearer ${$token}` }
            });
            if (res.ok) {
                roles = await res.json();
            } else {
                error = 'Failed to load roles';
            }
        } catch (e) {
            error = e.message;
        } finally {
            loading = false;
        }
    }

    function addPermission() {
        // Construct clean permission object based on scope type
        let scope: any = { type: newPerm.scopeType };
        if (newPerm.scopeType === 'Catalog') scope.catalog_id = newPerm.catalogId;
        if (newPerm.scopeType === 'Namespace') { scope.catalog_id = newPerm.catalogId; scope.namespace = newPerm.namespace; }
        if (newPerm.scopeType === 'Asset') { scope.catalog_id = newPerm.catalogId; scope.namespace = newPerm.namespace; scope.asset_id = newPerm.assetId; }
        if (newPerm.scopeType === 'Tag') { scope.tag_name = newPerm.tagName; }

        formRole.permissions = [...formRole.permissions, {
            scope,
            actions: [...newPerm.actions]
        }];
        
        // Reset (keep catalogId as it's often reused)
        newPerm.actions = [];
    }

    function removePermission(index: number) {
        formRole.permissions = formRole.permissions.filter((_, i) => i !== index);
    }

    function toggleAction(action: string) {
        if (newPerm.actions.includes(action)) {
            newPerm.actions = newPerm.actions.filter(a => a !== action);
        } else {
            newPerm.actions = [...newPerm.actions, action];
        }
    }

    async function handleSave() {
        // Logic for Create or Update
        try {
            const url = selectedRole ? `/api/v1/roles/${selectedRole.id}` : '/api/v1/roles';
            const method = selectedRole ? 'PUT' : 'POST';
            
            const res = await fetch(url, {
                method,
                headers: { 
                    'Authorization': `Bearer ${$token}`,
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify(formRole)
            });

            if (res.ok) {
                showCreate = false;
                showEdit = false;
                fetchRoles();
                resetForm();
            } else {
                alert('Failed to save role');
            }
        } catch (e) {
            console.error(e);
        }
    }

    async function handleDelete() {
        if (!selectedRole) return;
        try {
             const res = await fetch(`/api/v1/roles/${selectedRole.id}`, {
                method: 'DELETE',
                headers: { 'Authorization': `Bearer ${$token}` }
            });
            if (res.ok) {
                showDelete = false;
                fetchRoles();
            }
        } catch (e) {
            console.error(e);
        }
    }

    function openEdit(role: any) {
        selectedRole = role;
        formRole = JSON.parse(JSON.stringify(role)); // Deep copy
        showEdit = true;
    }

    function resetForm() {
        selectedRole = null;
        formRole = { name: '', permissions: [] };
        newPerm = { scopeType: 'Catalog', catalogId: '', namespace: '', assetId: '', tagName: '', actions: [] };
    }

    onMount(() => {
        fetchRoles();
    });
</script>

<div class="page-header">
    <h1>Roles</h1>
    <button class="primary-btn" on:click={() => { resetForm(); showCreate = true; }}>
        <span class="material-icons">add</span>
        Create Role
    </button>
</div>

{#if loading}
    <div class="loading">Loading roles...</div>
{:else if error}
    <div class="error">{error}</div>
{:else}
    <div class="grid">
        {#each roles as role}
            <div class="card">
                <div class="card-header">
                    <h3>{role.name}</h3>
                    <div class="actions">
                        <button class="icon-btn" on:click={() => openEdit(role)}>
                            <span class="material-icons">edit</span>
                        </button>
                        <button class="icon-btn delete" on:click={() => { selectedRole = role; showDelete = true; }}>
                            <span class="material-icons">delete</span>
                        </button>
                    </div>
                </div>
                <div class="card-body">
                    <p>{role.permissions?.length || 0} Permissions</p>
                    <div class="tags">
                        {#each role.permissions.slice(0, 3) as perm}
                            <span class="tag">{perm.scope.type}</span>
                        {/each}
                        {#if role.permissions.length > 3}
                            <span class="tag">+{role.permissions.length - 3}</span>
                        {/if}
                    </div>
                </div>
            </div>
        {/each}
    </div>
{/if}

<!-- Create/Edit Dialog -->
{#if showCreate || showEdit}
    <div class="modal-backdrop" transition:fade on:click={() => { showCreate=false; showEdit=false; }}>
        <div class="modal large" on:click|stopPropagation transition:scale>
            <h2>{selectedRole ? 'Edit Role' : 'Create Role'}</h2>
            
            <div class="form-group">
                <label>Role Name</label>
                <input bind:value={formRole.name} placeholder="e.g. Data Analyst" />
            </div>

            <div class="permissions-section">
                <h3>Permissions</h3>
                
                <!-- Permission Builder -->
                <div class="builder">
                    <div class="row">
                        <select bind:value={newPerm.scopeType}>
                            {#each SCOPES as scope}
                                <option value={scope}>{scope}</option>
                            {/each}
                        </select>
                        
                        {#if newPerm.scopeType === 'Catalog' || newPerm.scopeType === 'Namespace' || newPerm.scopeType === 'Asset'}
                            <input bind:value={newPerm.catalogId} placeholder="Catalog ID" />
                        {/if}
                        {#if newPerm.scopeType === 'Namespace' || newPerm.scopeType === 'Asset'}
                            <input bind:value={newPerm.namespace} placeholder="Namespace" />
                        {/if}
                        {#if newPerm.scopeType === 'Asset'}
                            <input bind:value={newPerm.assetId} placeholder="Asset ID" />
                        {/if}
                        {#if newPerm.scopeType === 'Tag'}
                            <input bind:value={newPerm.tagName} placeholder="Tag Name" />
                        {/if}
                    </div>
                    
                    <div class="actions-row">
                        {#each ACTIONS as action}
                            <button 
                                class="chip {newPerm.actions.includes(action) ? 'active' : ''}" 
                                on:click={() => toggleAction(action)}
                            >
                                {action}
                            </button>
                        {/each}
                        <button class="add-btn" on:click={addPermission} disabled={newPerm.actions.length === 0}>
                            Add Permission
                        </button>
                    </div>
                </div>

                <!-- Permission List -->
                <div class="perm-list">
                    {#each formRole.permissions as perm, i}
                        <div class="perm-item">
                            <div class="perm-info">
                                <strong>{perm.scope.type}</strong>
                                <small>{JSON.stringify(perm.scope)}</small>
                                <div class="perm-actions">
                                    {#each perm.actions as a}
                                        <span class="action-tag">{a}</span>
                                    {/each}
                                </div>
                            </div>
                            <button class="icon-btn delete" on:click={() => removePermission(i)}>
                                <span class="material-icons">close</span>
                            </button>
                        </div>
                    {/each}
                    {#if formRole.permissions.length === 0}
                        <div class="empty-state">No permissions added yet</div>
                    {/if}
                </div>
            </div>

            <div class="modal-actions">
                <button class="text-btn" on:click={() => { showCreate=false; showEdit=false; }}>Cancel</button>
                <button class="primary-btn" on:click={handleSave}>Save Role</button>
            </div>
        </div>
    </div>
{/if}

<style>
    .page-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 2rem;
    }

    h1 { margin: 0; font-size: 2rem; }

    .grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
        gap: 1.5rem;
    }

    .card {
        background-color: var(--md-sys-color-surface);
        border-radius: 16px;
        padding: 1.5rem;
        box-shadow: 0 1px 3px rgba(0,0,0,0.1);
    }

    .card-header {
        display: flex;
        justify-content: space-between;
        align-items: flex-start;
        margin-bottom: 1rem;
    }

    h3 { margin: 0; font-size: 1.25rem; }

    .tags {
        display: flex;
        flex-wrap: wrap;
        gap: 0.5rem;
        margin-top: 0.5rem;
    }

    .tag {
        background-color: var(--md-sys-color-surface-variant);
        color: var(--md-sys-color-on-surface-variant);
        padding: 4px 8px;
        border-radius: 4px;
        font-size: 0.75rem;
    }

    /* Reuse buttons from Users page... repeating styles for isolation */
     .primary-btn {
        background-color: var(--md-sys-color-primary);
        color: var(--md-sys-color-on-primary);
        border: none;
        padding: 0.75rem 1.5rem;
        border-radius: 100px;
        font-weight: 500;
        cursor: pointer;
        display: flex;
        align-items: center;
        gap: 0.5rem;
    }

    .icon-btn {
        background: none; border: none; cursor: pointer; padding: 4px; border-radius: 50%; color: inherit;
    }

    /* Modal */
    .modal-backdrop {
        position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0,0,0,0.5); display: flex; justify-content: center; align-items: center; z-index: 1000;
    }
    .modal {
        background: var(--md-sys-color-surface); padding: 2rem; border-radius: 28px; width: 90%; max-width: 500px;
        max-height: 90vh; overflow-y: auto;
    }
    .modal.large { max-width: 800px; }

    .form-group { margin-bottom: 1.5rem; }
    .form-group label { display: block; margin-bottom: 0.5rem; }
    .form-group input, select { width: 100%; padding: 0.75rem; border-radius: 4px; border: 1px solid var(--md-sys-color-outline); background: var(--md-sys-color-surface-container-highest); color: inherit; box-sizing: border-box; }

    /* Builder */
    .builder {
        background-color: var(--md-sys-color-surface-container);
        padding: 1rem;
        border-radius: 8px;
        margin-bottom: 1rem;
    }

    .row { display: flex; gap: 0.5rem; margin-bottom: 1rem; }
    .row input, .row select { flex: 1; }

    .actions-row { display: flex; flex-wrap: wrap; gap: 0.5rem; align-items: center; }
    
    .chip {
        background: var(--md-sys-color-surface);
        border: 1px solid var(--md-sys-color-outline);
        padding: 4px 12px;
        border-radius: 100px;
        cursor: pointer;
        color: inherit;
    }
    .chip.active {
        background: var(--md-sys-color-primary-container);
        color: var(--md-sys-color-on-primary-container);
        border-color: transparent;
    }

    .add-btn {
        margin-left: auto;
        background: var(--md-sys-color-primary);
        color: var(--md-sys-color-on-primary);
        border: none;
        padding: 8px 16px;
        border-radius: 100px;
        cursor: pointer;
    }
    .add-btn:disabled { opacity: 0.5; cursor: not-allowed; }

    .perm-list {
        display: flex; flex-direction: column; gap: 0.5rem;
    }

    .perm-item {
        display: flex; justify-content: space-between; align-items: flex-start;
        background: var(--md-sys-color-surface-container-low);
        padding: 0.75rem;
        border-radius: 8px;
    }
    
    .perm-info strong { display: block; }
    .perm-info small { opacity: 0.7; font-family: monospace; display: block; margin: 2px 0; }
    .perm-actions { display: flex; gap: 4px; margin-top: 4px; }
    .action-tag { font-size: 0.7rem; background: rgba(0,0,0,0.1); padding: 2px 4px; border-radius: 3px; }

    .modal-actions { display: flex; justify-content: flex-end; gap: 1rem; margin-top: 2rem; }
    .text-btn { background: none; border: none; color: var(--md-sys-color-primary); cursor: pointer; font-weight: 500; }
</style>
