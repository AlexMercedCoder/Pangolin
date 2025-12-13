<script lang="ts">
    import { onMount } from 'svelte';
    import { user, token } from '$lib/auth';
    import { fade, scale } from 'svelte/transition';

    let users: any[] = [];
    let loading = true;
    let error = '';

    // Role Assignment State
    let showRoles = false;
    let availableRoles: any[] = [];
    let userPermissions: any[] = [];
    let roleLoading = false;

    async function openRoles(u: any) {
        selectedUser = u;
        showRoles = true;
        roleLoading = true;
        try {
            await Promise.all([
                fetchRoles(),
                fetchUserPermissions(u.id)
            ]);
        } finally {
            roleLoading = false;
        }
    }

    async function fetchRoles() {
        const res = await fetch('/api/v1/roles', { headers: { 'Authorization': `Bearer ${$token}` } });
        if (res.ok) availableRoles = await res.json();
    }

    async function fetchUserPermissions(userId: string) {
        const res = await fetch(`/api/v1/permissions/user/${userId}`, { headers: { 'Authorization': `Bearer ${$token}` } });
        if (res.ok) userPermissions = await res.json();
    }

    async function assignRole(roleId: string) {
        try {
            const res = await fetch(`/api/v1/users/${selectedUser.id}/roles`, {
                method: 'POST',
                headers: { 
                    'Authorization': `Bearer ${$token}`,
                    'Content-Type': 'application/json' 
                },
                body: JSON.stringify({ role_id: roleId }) // Adjust body based on API expectation
            });
            if (res.ok) {
                // Refresh permissions
                fetchUserPermissions(selectedUser.id);
            }
        } catch(e) { console.error(e); }
    }

    async function removeRole(roleId: string) {
        // API might be DELETE /users/{id}/roles/{roleId}
        try {
            const res = await fetch(`/api/v1/users/${selectedUser.id}/roles/${roleId}`, {
                method: 'DELETE',
                headers: { 'Authorization': `Bearer ${$token}` }
            });
             if (res.ok) {
                fetchUserPermissions(selectedUser.id);
            }
        } catch(e) { console.error(e); }
    }
</script>

<!-- ... existing header ... -->

<!-- Update Actions Column -->
<!-- In the table loop -->
                        <td class="actions">
                            <button class="icon-btn" on:click={() => openRoles(u)} title="Manage Roles">
                                <span class="material-icons">admin_panel_settings</span>
                            </button>
                            <button class="icon-btn" on:click={() => openEdit(u)} title="Edit">
                                <span class="material-icons">edit</span>
                            </button>
                            <!-- ... delete btn ... -->
                        </td>

<!-- ... existing dialogs ... -->

<!-- Manage Roles Dialog -->
{#if showRoles}
    <div class="modal-backdrop" transition:fade on:click={() => showRoles = false}>
        <div class="modal large" on:click|stopPropagation transition:scale>
            <h2>Manage Roles: {selectedUser?.username}</h2>
            
            <div class="split-view">
                <div class="panel">
                    <h3>Assigned Roles & Permissions</h3>
                    {#if roleLoading}
                        <div>Loading...</div>
                    {:else}
                         <!-- Currently API might not return assigned roles directly in user object, 
                              so we might infer from permissions or checking a 'user_roles' endpoint.
                              For now let's assume we can see permissions. -->
                        <div class="perm-list">
                            {#each userPermissions as perm}
                                <div class="perm-item">
                                    <div>
                                        <strong>{perm.scope.type}</strong>
                                        <div class="actions-list">
                                            {#each perm.actions as a}
                                                <span class="action-tag">{a}</span>
                                            {/each}
                                        </div>
                                    </div>
                                    <!-- Direct permission revoke? -->
                                </div>
                            {/each}
                            {#if userPermissions.length === 0}
                                <p class="empty">No active permissions</p>
                            {/if}
                        </div>
                    {/if}
                </div>

                <div class="panel">
                    <h3>Available Roles</h3>
                    <div class="role-list">
                         {#each availableRoles as role}
                            <div class="role-item">
                                <div>
                                    <strong>{role.name}</strong>
                                    <small>{role.permissions.length} perms</small>
                                </div>
                                <button class="primary-btn small" on:click={() => assignRole(role.id)}>Assign</button>
                            </div>
                         {/each}
                    </div>
                </div>
            </div>

            <div class="modal-actions">
                <button class="text-btn" on:click={() => showRoles = false}>Close</button>
            </div>
        </div>
    </div>
{/if}

<style>
    /* ... existing styles ... */
    
    .role-list, .perm-list { display: flex; flex-direction: column; gap: 0.5rem; }
    .role-item, .perm-item { 
        padding: 0.75rem; background: var(--md-sys-color-surface-container-low); border-radius: 8px;
        display: flex; justify-content: space-between; align-items: center;
    }
    .primary-btn.small { padding: 4px 12px; font-size: 0.75rem; }
    
    .split-view { display: grid; grid-template-columns: 1fr 1fr; gap: 2rem; }
    .panel h3 { margin-top: 0; border-bottom: 1px solid var(--md-sys-color-outline-variant); padding-bottom: 0.5rem; }
    
    .actions-list { display: flex; gap: 4px; flex-wrap: wrap; margin-top: 4px; }
    .action-tag { font-size: 0.7rem; background: rgba(0,0,0,0.1); padding: 2px 4px; border-radius: 2px; }
    .empty { opacity: 0.5; font-style: italic; }
</style>

    // Form Data
    let formData = {
        username: '',
        email: '',
        password: '', // Only for create
        role: 'TenantUser', // Default
        tenant_id: '' // For root user
    };

    async function fetchUsers() {
        loading = true;
        try {
            const res = await fetch('/api/v1/users', {
                headers: { 'Authorization': `Bearer ${$token}` }
            });
            if (res.ok) {
                users = await res.json();
            } else {
                error = 'Failed to load users';
            }
        } catch (e) {
            error = e.message;
        } finally {
            loading = false;
        }
    }

    async function handleCreate() {
        try {
            const res = await fetch('/api/v1/users', {
                method: 'POST',
                headers: { 
                    'Authorization': `Bearer ${$token}`,
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify(formData)
            });
            if (res.ok) {
                showCreate = false;
                fetchUsers();
                resetForm();
            } else {
                alert('Failed to create user');
            }
        } catch (e) {
            console.error(e);
        }
    }
    
    async function handleDelete() {
        if (!selectedUser) return;
        try {
            const res = await fetch(`/api/v1/users/${selectedUser.id}`, {
                method: 'DELETE',
                headers: { 'Authorization': `Bearer ${$token}` }
            });
            if (res.ok) {
                showDelete = false;
                fetchUsers();
            }
        } catch (e) {
            console.error(e);
        }
    }

    function openEdit(u: any) {
        selectedUser = u;
        formData = { ...u, password: '' }; // Don't allow password edit here usually? Or optional
        showEdit = true;
    }
    
    function resetForm() {
        formData = { username: '', email: '', password: '', role: 'TenantUser', tenant_id: '' };
    }

    onMount(() => {
        fetchUsers();
    });
</script>

<div class="page-header">
    <h1>Users</h1>
    <button class="primary-btn" on:click={() => { resetForm(); showCreate = true; }}>
        <span class="material-icons">add</span>
        Add User
    </button>
</div>

{#if loading}
    <div class="loading">Loading users...</div>
{:else if error}
    <div class="error">{error}</div>
{:else}
    <div class="table-container">
        <table>
            <thead>
                <tr>
                    <th>Avatar</th>
                    <th>Username</th>
                    <th>Email</th>
                    <th>Role</th>
                    <th>Actions</th>
                </tr>
            </thead>
            <tbody>
                {#each users as u}
                    <tr>
                        <td>
                            <div class="avatar">{u.username.charAt(0).toUpperCase()}</div>
                        </td>
                        <td>{u.username}</td>
                        <td>{u.email || '-'}</td>
                        <td><span class="role-badge">{u.role}</span></td>
                        <td class="actions">
                            <button class="icon-btn" on:click={() => openEdit(u)} title="Edit">
                                <span class="material-icons">edit</span>
                            </button>
                            <button class="icon-btn delete" on:click={() => { selectedUser = u; showDelete = true; }} title="Delete">
                                <span class="material-icons">delete</span>
                            </button>
                        </td>
                    </tr>
                {/each}
            </tbody>
        </table>
    </div>
{/if}

<!-- Create Dialog -->
{#if showCreate}
    <div class="modal-backdrop" transition:fade on:click={() => showCreate = false}>
        <div class="modal" on:click|stopPropagation transition:scale>
            <h2>Create User</h2>
            <form on:submit|preventDefault={handleCreate}>
                <div class="form-group">
                    <label>Username</label>
                    <input bind:value={formData.username} required />
                </div>
                <div class="form-group">
                    <label>Password</label>
                    <input type="password" bind:value={formData.password} required />
                </div>
                <div class="form-group">
                    <label>Role</label>
                    <select bind:value={formData.role}>
                        <option value="TenantUser">Tenant User</option>
                        <option value="TenantAdmin">Tenant Admin</option>
                    </select>
                </div>
                <!-- TODO: Role assignment UI separately or here? -->
                
                <div class="modal-actions">
                    <button type="button" class="text-btn" on:click={() => showCreate = false}>Cancel</button>
                    <button type="submit" class="primary-btn">Create</button>
                </div>
            </form>
        </div>
    </div>
{/if}

<!-- Delete Confirmation -->
{#if showDelete}
    <div class="modal-backdrop" transition:fade on:click={() => showDelete = false}>
        <div class="modal" on:click|stopPropagation transition:scale>
            <h2>Delete User?</h2>
            <p>Are you sure you want to delete <strong>{selectedUser?.username}</strong>? This action cannot be undone.</p>
            <div class="modal-actions">
                <button class="text-btn" on:click={() => showDelete = false}>Cancel</button>
                <button class="primary-btn delete" on:click={handleDelete}>Delete</button>
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

    h1 {
        font-size: 2rem;
        margin: 0;
        color: var(--md-sys-color-on-background);
    }

    .table-container {
        background-color: var(--md-sys-color-surface);
        border-radius: 16px;
        overflow: hidden;
        box-shadow: 0 1px 3px rgba(0,0,0,0.1);
    }

    table {
        width: 100%;
        border-collapse: collapse;
    }

    th, td {
        padding: 1rem 1.5rem;
        text-align: left;
        border-bottom: 1px solid rgba(0,0,0,0.05);
    }

    th {
        font-weight: 500;
        color: var(--md-sys-color-on-surface-variant);
        font-size: 0.875rem;
    }

    tr:last-child td {
        border-bottom: none;
    }
    
    .avatar {
        width: 32px;
        height: 32px;
        background-color: var(--md-sys-color-primary-container);
        color: var(--md-sys-color-on-primary-container);
        border-radius: 50%;
        display: flex;
        justify-content: center;
        align-items: center;
        font-weight: bold;
    }

    .role-badge {
        background-color: var(--md-sys-color-secondary-container);
        color: var(--md-sys-color-on-secondary-container);
        padding: 4px 8px;
        border-radius: 4px;
        font-size: 0.75rem;
        font-weight: 500;
    }

    .actions {
        display: flex;
        gap: 0.5rem;
    }

    .icon-btn {
        background: none;
        border: none;
        cursor: pointer;
        padding: 4px;
        border-radius: 50%;
        color: var(--md-sys-color-on-surface-variant);
        transition: background-color 0.2s;
    }

    .icon-btn:hover {
        background-color: rgba(0,0,0,0.05);
    }

    .icon-btn.delete:hover {
        color: var(--md-sys-color-error);
        background-color: var(--md-sys-color-error-container);
    }

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
        transition: opacity 0.2s;
    }

    .primary-btn:hover {
        opacity: 0.9;
    }
    
    .primary-btn.delete {
        background-color: var(--md-sys-color-error);
        color: var(--md-sys-color-on-error);
    }

    .text-btn {
        background: none;
        border: none;
        color: var(--md-sys-color-primary);
        padding: 0.75rem 1.5rem;
        border-radius: 100px;
        font-weight: 500;
        cursor: pointer;
    }
    
    .text-btn:hover {
        background-color: rgba(0,0,0,0.05);
    }

    /* Modal Styles */
    .modal-backdrop {
        position: fixed;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        background-color: rgba(0,0,0,0.5);
        display: flex;
        justify-content: center;
        align-items: center;
        z-index: 1000;
    }

    .modal {
        background-color: var(--md-sys-color-surface);
        padding: 2rem;
        border-radius: 28px;
        width: 100%;
        max-width: 400px;
        box-shadow: 0 4px 6px rgba(0,0,0,0.1);
    }

    .modal h2 {
        margin-top: 0;
    }

    .form-group {
        margin-bottom: 1rem;
    }

    .form-group label {
        display: block;
        margin-bottom: 0.5rem;
        font-size: 0.875rem;
        color: var(--md-sys-color-on-surface-variant);
    }

    .form-group input, .form-group select {
        width: 100%;
        padding: 0.75rem;
        border-radius: 4px;
        border: 1px solid var(--md-sys-color-outline);
        background: var(--md-sys-color-surface-container-highest); /* M3 typical input bg */
        color: var(--md-sys-color-on-surface);
        font-size: 1rem;
        box-sizing: border-box;
    }

    .modal-actions {
        display: flex;
        justify-content: flex-end;
        gap: 1rem;
        margin-top: 2rem;
    }
</style>
