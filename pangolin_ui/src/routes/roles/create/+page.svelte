<script lang="ts">
  import { goto } from '$app/navigation';
	import { rolesApi, type Role } from '$lib/api/roles';
	import Button from '$lib/components/ui/Button.svelte';
	import Card from '$lib/components/ui/Card.svelte';
	import { notifications } from '$lib/stores/notifications';
	import PermissionBuilder from '$lib/components/rbac/PermissionBuilder.svelte';
    import PermissionHelpTable from '$lib/components/rbac/PermissionHelpTable.svelte';
	import type { ScopeType, ActionType } from '$lib/components/rbac/PermissionBuilder.svelte';
    import { tenantStore } from '$lib/stores/tenant';
    import { authStore } from '$lib/stores/auth';
  
  let name = '';
  let description = '';
  let permissions: { scope: { type: ScopeType, id?: string }, actions: ActionType[] }[] = [];
  let submitting = false;

  // Temporary holder for the permission being built
  let currentScope: { type: ScopeType, id?: string } = { type: 'Tenant' };
  let currentActions: ActionType[] = [];

  function handlePermissionChange(event: CustomEvent) {
    currentScope = event.detail.scope;
    currentActions = event.detail.actions;
  }

  function addPermission() {
    if (currentActions.length === 0) {
      notifications.error('Please select at least one action.');
      return;
    }
    // minimal validation for scope id presence if needed
    
    permissions = [...permissions, {
      scope: { ...currentScope },
      actions: [...currentActions]
    }];
    
    // reset builder slightly or keep for ease of bulk add? Let's keep for now.
    currentActions = [];
    notifications.success('Permission added to list');
  }

  function removePermission(index: number) {
    permissions = permissions.filter((_, i) => i !== index);
  }

  import { catalogsApi, type Catalog } from '$lib/api/catalogs';
  import { onMount } from 'svelte';

  let catalogs: Catalog[] = [];

  onMount(async () => {
      try {
          catalogs = await catalogsApi.list();
      } catch (e) {
          console.error('Failed to load catalogs', e);
      }
  });

  async function handleSubmit() {
    if (!name.trim()) return notifications.error('Role Name is required');
    if (permissions.length === 0) return notifications.error('Please add at least one permission');

    submitting = true;
    try {
      // 1. Get Tenant ID
      const tenantId = $tenantStore.selectedTenantId || $authStore.user?.tenant_id;
      if (!tenantId) {
          throw new Error('Tenant Context missing. Cannot create role.');
      }

      // 2. Create Role
      const role = await rolesApi.create({
        name,
        description,
        'tenant-id': tenantId
      });

      const backendPermissions = [];
      
      for (const p of permissions) {
          let backendScope: any;
          const resourceName = p.scope.id || '';
          
          if (p.scope.type === 'Catalog') {
              // Find catalog by name
              const catalog = catalogs.find(c => c.name === resourceName);
              if (!catalog) {
                   notifications.error(`Catalog '${resourceName}' not found. Please verify the name.`);
                   submitting = false;
                   return;
              } else {
                   backendScope = { type: 'catalog', 'catalog_id': catalog.id };
              }
          } else if (p.scope.type === 'Namespace') {
              // Expecting format "catalogName.namespaceName"
              const parts = resourceName.split('.');
              if (parts.length >= 2) {
                  const catName = parts[0];
                  // join the rest in case namespace has dots? assume 1st dot is sep
                  const nsName = parts.slice(1).join('.'); 
                  
                  const catalog = catalogs.find(c => c.name === catName);
                  if (catalog) {
                      backendScope = { 
                          type: 'namespace', 
                          'catalog_id': catalog.id,
                          'namespace': nsName
                      };
                  } else {
                      notifications.error(`Catalog '${catName}' not found in namespace '${resourceName}'.`);
                      submitting = false;
                      return;
                  }
              } else {
                  notifications.error(`Invalid Namespace format '${resourceName}'. Use 'catalog.namespace'.`);
                  submitting = false;
                  return;
              }
          } else if (p.scope.type === 'Tenant') {
              backendScope = { type: 'tenant' };
          } else {
              // Default / Asset (Asset needs ID too, which is hard. Not fixing Asset right now as per user flow)
               backendScope = { type: 'catalog', 'catalog_id': p.scope.id };
          }

          backendPermissions.push({
              scope: backendScope,
              actions: p.actions.map(a => a.toLowerCase())
          });
      }
      
      role.permissions = backendPermissions;
      console.log('Sending Role Update with Permissions:', JSON.stringify(role, null, 2));

      await rolesApi.update(role.id, role);

      notifications.success('Role created successfully');
      goto('/roles');
    } catch (error: any) {
      console.error('Role Creation Error:', error);
      console.error('Role Creation Error Message:', error.message);
      notifications.error(`Failed to create role: ${error.message}`);
    }
    submitting = false;
  }
</script>

<svelte:head>
  <title>Create Role - Pangolin</title>
</svelte:head>

<div class="max-w-4xl mx-auto space-y-6">
  <div class="flex items-center justify-between">
    <h1 class="text-3xl font-bold text-gray-900 dark:text-white">Create Role</h1>
  </div>

  <Card title="Role Details">
    <div class="space-y-4">
      <div>
        <label class="label mb-1 text-gray-700 dark:text-gray-300" for="role-name">Role Name</label>
        <input 
            type="text" 
            id="role-name"
            class="input w-full text-gray-900 dark:text-white bg-white dark:bg-gray-800" 
            bind:value={name} 
            placeholder="e.g. Data Analyst"
        />
      </div>
      <div>
        <label class="label mb-1" for="role-description">Description (Optional)</label>
        <textarea 
            id="role-description"
            class="textarea w-full text-gray-900 dark:text-white bg-white dark:bg-gray-800" 
            bind:value={description} 
            placeholder="Role description..."
        ></textarea>
      </div>
    </div>
  </Card>

  <Card title="Grant Permissions">
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

      <!-- List of Added Permissions -->
      <div class="border-t border-surface-600 pt-4">
        <h3 class="font-bold mb-2">Staged Permissions</h3>
        {#if permissions.length === 0}
            <p class="text-surface-400 italic text-sm">No permissions added yet.</p>
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

  <div class="flex justify-end gap-2">
    <Button variant="secondary" on:click={() => goto('/roles')}>Cancel</Button>
    <Button on:click={handleSubmit} disabled={submitting}>
      {submitting ? 'Creating...' : 'Create Role'}
    </Button>
  </div>
</div>
