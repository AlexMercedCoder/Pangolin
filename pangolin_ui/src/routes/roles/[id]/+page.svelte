<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { rolesApi, type Role } from '$lib/api/roles';
  import Button from '$lib/components/ui/Button.svelte';
  import Card from '$lib/components/ui/Card.svelte';
  import { notifications } from '$lib/stores/notifications';
  import PermissionBuilder from '$lib/components/rbac/PermissionBuilder.svelte';
  import type { ScopeType, ActionType } from '$lib/components/rbac/PermissionBuilder.svelte';

  let role: Role | null = null;
  let loading = true;
  let submitting = false;

  // Edit State
  let name = '';
  let description = '';
  let permissions: { scope: { type: ScopeType, id?: string }, actions: ActionType[] }[] = [];

  // Builder State
  let currentScope: { type: ScopeType, id?: string } = { type: 'Tenant' };
  let currentActions: ActionType[] = [];

  onMount(async () => {
    await loadRole();
  });

  async function loadRole() {
    loading = true;
    try {
      const id = $page.params.id;
      role = await rolesApi.get(id);
      name = role.name;
      description = role.description || '';
      // Parse permissions if they come as object from API (Role interface has any[])
      // In create we sent JSON string. The API get likely returns objects or the string.
      // Assuming API returns parsed JSON array of objects matching our structure.
      if (Array.isArray(role.permissions)) {
        permissions = role.permissions;
      } else if (typeof role.permissions === 'string') {
          try {
              permissions = JSON.parse(role.permissions);
          } catch (e) {
              permissions = [];
          }
      }
    } catch (error: any) {
      notifications.error(`Failed to load role: ${error.message}`);
      goto('/roles');
    }
    loading = false;
  }

  function handlePermissionChange(event: CustomEvent) {
    currentScope = event.detail.scope;
    currentActions = event.detail.actions;
  }

  function addPermission() {
    if (currentActions.length === 0) {
      notifications.error('Please select at least one action.');
      return;
    }
    permissions = [...permissions, {
      scope: { ...currentScope },
      actions: [...currentActions]
    }];
    currentActions = [];
    notifications.success('Permission added');
  }

  function removePermission(index: number) {
    permissions = permissions.filter((_, i) => i !== index);
  }

  async function handleSave() {
    if (!role) return;
    submitting = true;
    try {
      await rolesApi.update(role.id, {
        name,
        description,
        permissions: JSON.stringify(permissions)
      });
      notifications.success('Role updated successfully');
    } catch (error: any) {
      notifications.error(`Failed to update role: ${error.message}`);
    }
    submitting = false;
  }

  async function handleDelete() {
      if (!role) return;
      if (!confirm('Are you sure you want to delete this role? This cannot be undone.')) return;
      
      try {
          await rolesApi.delete(role.id);
          notifications.success('Role deleted');
          goto('/roles');
      } catch (error: any) {
          notifications.error(`Failed to delete role: ${error.message}`);
      }
  }
</script>

<svelte:head>
  <title>Edit Role - Pangolin</title>
</svelte:head>

<div class="max-w-4xl mx-auto space-y-6">
  <div class="flex items-center justify-between">
    <h1 class="text-3xl font-bold">Edit Role</h1>
    {#if role}
      <p class="text-surface-400 font-mono text-sm">ID: {role.id}</p>
    {/if}
  </div>

  {#if loading}
    <div class="flex justify-center p-8">
      <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-500"></div>
    </div>
  {:else if role}
    <Card title="Role Details">
      <div class="space-y-4">
        <div>
          <label class="label mb-1">Role Name</label>
          <input 
              type="text" 
              class="input w-full" 
              bind:value={name} 
          />
        </div>
        <div>
          <label class="label mb-1">Description</label>
          <textarea 
              class="textarea w-full" 
              bind:value={description} 
          ></textarea>
        </div>
      </div>
    </Card>

    <Card title="Manage Permissions">
      <div class="space-y-6">
        <PermissionBuilder 
          scope={currentScope} 
          actions={currentActions} 
          on:change={handlePermissionChange} 
        />
        <div class="flex justify-end">
          <Button variant="secondary" on:click={addPermission}>
            Add Permission
          </Button>
        </div>

        <div class="border-t border-surface-600 pt-4">
          <h3 class="font-bold mb-2">Current Permissions</h3>
          {#if permissions.length === 0}
              <p class="text-surface-400 italic text-sm">No permissions assigned.</p>
          {:else}
              <div class="space-y-2">
                  {#each permissions as perm, i}
                      <div class="p-3 bg-surface-700 rounded flex justify-between items-center">
                          <div>
                              <span class="font-mono font-bold text-primary-400">{perm.scope.type}</span>
                              {#if perm.scope.id} 
                                  <span class="mx-1 text-surface-400">/</span> 
                                  <span class="font-mono text-secondary-400">{perm.scope.id}</span> 
                              {/if}
                              <div class="flex flex-wrap gap-1 mt-1">
                                  {#each perm.actions as action}
                                      <span class="badge variant-soft-secondary text-xs">{action}</span>
                                  {/each}
                              </div>
                          </div>
                          <button 
                              class="btn-icon btn-icon-sm variant-soft-error"
                              on:click={() => removePermission(i)}
                          >
                              ✕
                          </button>
                      </div>
                  {/each}
              </div>
          {/if}
        </div>
      </div>
    </Card>

    <div class="flex justify-between">
        <Button variant="error" on:click={handleDelete}>Delete Role</Button>
        <div class="flex gap-2">
            <Button variant="secondary" on:click={() => goto('/roles')}>Cancel</Button>
            <Button on:click={handleSave} disabled={submitting}>
                {submitting ? 'Saving...' : 'Save Changes'}
            </Button>
        </div>
    </div>
  {:else}
    <div class="alert variant-soft-error">Role not found</div>
  {/if}
</div>
