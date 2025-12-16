<script lang="ts">
  import { goto } from '$app/navigation';
  import { rolesApi, type Role } from '$lib/api/roles';
  import Button from '$lib/components/ui/Button.svelte';
  import Card from '$lib/components/ui/Card.svelte';
  import { notifications } from '$lib/stores/notifications';
  import PermissionBuilder from '$lib/components/rbac/PermissionBuilder.svelte';
  import type { ScopeType, ActionType } from '$lib/components/rbac/PermissionBuilder.svelte';
  
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

  async function handleSubmit() {
    if (!name.trim()) return notifications.error('Role Name is required');
    if (permissions.length === 0) return notifications.error('Please add at least one permission');

    submitting = true;
    try {
      // Construct Role object
      // Note: Backend might expect specific JSON structure for permissions
      await rolesApi.create({
        name,
        description,
        permissions: JSON.stringify(permissions) // Assuming backend takes serialized JSON or similar
      });
      notifications.success('Role created successfully');
      goto('/roles');
    } catch (error: any) {
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
    <h1 class="text-3xl font-bold">Create Role</h1>
  </div>

  <Card title="Role Details">
    <div class="space-y-4">
      <div>
        <label class="label mb-1">Role Name</label>
        <input 
            type="text" 
            class="input w-full" 
            bind:value={name} 
            placeholder="e.g. Data Analyst"
        />
      </div>
      <div>
        <label class="label mb-1">Description (Optional)</label>
        <textarea 
            class="textarea w-full" 
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
