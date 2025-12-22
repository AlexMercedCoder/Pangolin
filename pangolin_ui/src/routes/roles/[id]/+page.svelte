<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { rolesApi, type Role } from '$lib/api/roles';
  import { catalogsApi, type Catalog } from '$lib/api/catalogs';
  import Button from '$lib/components/ui/Button.svelte';
  import Card from '$lib/components/ui/Card.svelte';
  import { notifications } from '$lib/stores/notifications';
  import PermissionBuilder from '$lib/components/rbac/PermissionBuilder.svelte';
  import PermissionHelpTable from '$lib/components/rbac/PermissionHelpTable.svelte';
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

  let catalogs: Catalog[] = [];

  onMount(async () => {
    // Load catalogs FIRST so we can resolve IDs to names when loading the role
    await loadCatalogs();
    await loadRole();
  });

  async function loadCatalogs() {
      try {
          catalogs = await catalogsApi.list();
      } catch (e) {
          console.error('Failed to load catalogs', e);
      }
  }

  async function loadRole() {
    loading = true;
    try {
      const id = $page.params.id || '';
      if (!id) return;
      role = await rolesApi.get(id);
      name = role.name;
      description = role.description || '';
      // Parse permissions if they come as object from API
      let rawPermissions: any[] = [];
      if (Array.isArray(role.permissions)) {
        rawPermissions = role.permissions;
      } else if (typeof role.permissions === 'string') {
          try {
              rawPermissions = JSON.parse(role.permissions);
          } catch (e) {
              rawPermissions = [];
          }
      }
      
      // Normalize permissions for UI
      permissions = rawPermissions.map((p: any) => {
          let scopeType: ScopeType = 'Catalog'; // Default
          let scopeId = '';
          
          if (p.scope) {
              const rawType = p.scope.type?.toLowerCase();
              if (rawType === 'catalog') scopeType = 'Catalog';
              else if (rawType === 'namespace') scopeType = 'Namespace';
              else if (rawType === 'table') scopeType = 'Table';
              else if (rawType === 'view') scopeType = 'View';
              else if (rawType === 'tenant') scopeType = 'Tenant';
              else if (rawType === 'system') scopeType = 'System';
              
              // Helper to resolve catalog name from ID
              const resolveCatalogName = (id: string) => {
                  const cat = catalogs.find(c => c.id === id);
                  return cat ? cat.name : id;
              };

              // Reconstruct "catalogName.namespaceName" format for UI
              if (scopeType === 'Catalog') {
                  const id = p.scope['catalog-id'] || p.scope.catalog_id;
                  scopeId = resolveCatalogName(id);
              } 
              else if (scopeType === 'Namespace') {
                  const catId = p.scope['catalog-id'] || p.scope.catalog_id;
                  const nsIs = p.scope['namespace'];
                  const catName = resolveCatalogName(catId);
                  scopeId = `${catName}.${nsIs}`;
              }
              else if (scopeType === 'Table' || scopeType === 'View') {
                  const catId = p.scope['catalog-id'] || p.scope.catalog_id;
                  const nsIs = p.scope['namespace'];
                  const tblIs = p.scope['table'] || p.scope['view'];
                  const catName = resolveCatalogName(catId);
                  scopeId = `${catName}.${nsIs}.${tblIs}`;
              }
              else if (scopeType === 'Tenant' || scopeType === 'System') {
                  scopeId = ''; // No ID needed
              }
              else {
                  // Fallback
                  scopeId = p.scope['catalog-id'] || p.scope.catalog_id || '';
              }
          }
          
          return {
              scope: { type: scopeType, id: scopeId },
              actions: p.actions || []
          };
      });
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
      // Logic mirrored from roles/create/+page.svelte to fix 422 error
      const backendPermissions = [];
      
      for (const p of permissions) {
          let backendScope: any;
          const resourceName = p.scope.id || '';
          
          if (p.scope.type === 'Catalog') {
              const catalog = catalogs.find(c => c.name === resourceName);
              if (!catalog) {
                   notifications.error(`Catalog '${resourceName}' not found. Please verify the name.`);
                   submitting = false;
                   return; // STOP SAVE
              } else {
                   backendScope = { type: 'catalog', 'catalog_id': catalog.id };
              }
          } else if (p.scope.type === 'Namespace') {
              const parts = resourceName.split('.');
              if (parts.length >= 2) {
                  const catName = parts[0];
                  const nsName = parts.slice(1).join('.'); 
                  const catalog = catalogs.find(c => c.name === catName);
                  if (catalog) {
                      backendScope = { type: 'namespace', 'catalog_id': catalog.id, 'namespace': nsName };
                  } else {
                      notifications.error(`Catalog '${catName}' not found in namespace '${resourceName}'.`);
                      submitting = false;
                      return; // STOP SAVE
                  }
              } else {
                 notifications.error(`Invalid Namespace format '${resourceName}'. Use 'catalog.namespace'.`);
                 submitting = false;
                 return; // STOP SAVE
              }
          } else if (p.scope.type === 'Tenant') {
              backendScope = { type: 'tenant' };
          } else {
               backendScope = { type: 'catalog', 'catalog_id': p.scope.id };
          }

          backendPermissions.push({
              scope: backendScope,
              actions: p.actions.map(a => a.toLowerCase())
          });
      }

      // Update local role object first
      role.name = name;
      role.description = description;
      role.permissions = backendPermissions;

      // Send FULL role object
      await rolesApi.update(role.id, role);
      
      notifications.success('Role updated successfully');
      // Reload to ensure state sync?
      await loadRole();
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
          <label for="role-name" class="label mb-1">Role Name</label>
          <input 
              id="role-name"
              type="text" 
              class="input w-full text-gray-900 dark:text-white bg-white dark:bg-gray-800"
              bind:value={name} 
          />
        </div>
        <div>
          <label for="role-description" class="label mb-1">Description</label>
          <textarea 
              id="role-description"
              class="textarea w-full text-gray-900 dark:text-white bg-white dark:bg-gray-800"
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
        
        <div class="mt-4">
            <PermissionHelpTable />
        </div>

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
