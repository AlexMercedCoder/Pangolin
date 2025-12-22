<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  
  export let availableRoles: { id: string; name: string; description?: string }[] = [];
  export let assignedRoleIds: string[] = [];

  const dispatch = createEventDispatcher();

  function toggleRole(roleId: string) {
    if (assignedRoleIds.includes(roleId)) {
      assignedRoleIds = assignedRoleIds.filter(id => id !== roleId);
    } else {
      assignedRoleIds = [...assignedRoleIds, roleId];
    }
    dispatch('change', { assignedRoleIds });
  }

  $: unassignedRoles = availableRoles.filter(r => !assignedRoleIds.includes(r.id));
  $: assignedRolesList = availableRoles.filter(r => assignedRoleIds.includes(r.id));

</script>

<div class="grid grid-cols-1 md:grid-cols-2 gap-6 h-96">
  <!-- Available Roles -->
  <div class="card p-4 flex flex-col h-full">
    <header class="card-header pb-2 font-bold text-lg border-b border-surface-600 mb-2">
      Available Roles
    </header>
    <div class="flex-1 overflow-y-auto space-y-2">
        <p class="text-xs text-surface-400 px-2">Click to assign:</p>
        {#if unassignedRoles.length === 0}
            <p class="text-surface-400 italic text-sm p-2">No more roles available.</p>
        {/if}
      {#each unassignedRoles as role}
        <button 
          class="w-full text-left p-2 rounded hover:bg-surface-700 flex justify-between items-center border border-surface-600"
          on:click={() => toggleRole(role.id)}
        >
          <div>
            <div class="font-medium">{role.name}</div>
            {#if role.description}
                <div class="text-xs text-surface-400">{role.description}</div>
            {/if}
          </div>
          <span class="badge variant-soft-primary">
            + Assign
          </span>
        </button>
      {/each}
    </div>
  </div>

  <!-- Assigned Roles -->
  <div class="card p-4 flex flex-col h-full bg-surface-800">
    <header class="card-header pb-2 font-bold text-lg border-b border-surface-600 mb-2">
      Assigned Roles
    </header>
    <div class="flex-1 overflow-y-auto space-y-2">
        <p class="text-xs text-surface-400 px-2">Click to remove:</p>
        {#if assignedRolesList.length === 0}
            <p class="text-surface-400 italic text-sm p-2">No roles assigned yet.</p>
        {/if}
      {#each assignedRolesList as role}
        <button 
          class="w-full text-left p-2 rounded hover:bg-surface-700 flex justify-between items-center border border-surface-600"
          on:click={() => toggleRole(role.id)}
        >
          <div>
            <div class="font-medium text-primary-400">{role.name}</div>
             {#if role.description}
                <div class="text-xs text-surface-400">{role.description}</div>
            {/if}
          </div>
          <span class="badge variant-soft-error">
            REMOVE
          </span>
        </button>
      {/each}
    </div>
  </div>
</div>
