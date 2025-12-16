<script context="module" lang="ts">
  // Types based on backend Permission struct
  export type ScopeType = 'System' | 'Tenant' | 'Catalog' | 'Namespace' | 'Table' | 'View';
  export type ActionType = 'READ' | 'WRITE' | 'CREATE' | 'DELETE' | 'ADMIN' | 'MANAGE_ACCESS' | 'MANAGE_DISCOVERY';
</script>

<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  
  // Re-export or use directly (in Svelte <script> they are available)
  // We don't need to re-define types but we need variables using them.


  export let scope: { type: ScopeType; id?: string } = { type: 'Tenant' };
  export let actions: ActionType[] = [];
  
  const dispatch = createEventDispatcher();

  const scopeOptions: ScopeType[] = ['System', 'Tenant', 'Catalog', 'Namespace', 'Table', 'View'];
  const actionOptions: ActionType[] = ['READ', 'WRITE', 'CREATE', 'DELETE', 'ADMIN', 'MANAGE_ACCESS', 'MANAGE_DISCOVERY'];

  function toggleAction(action: ActionType) {
    if (actions.includes(action)) {
      actions = actions.filter(a => a !== action);
    } else {
      actions = [...actions, action];
    }
    dispatch('change', { scope, actions });
  }

  function updateScopeType(e: Event) {
    const target = e.target as HTMLSelectElement;
    scope.type = target.value as ScopeType;
    // reset id when type changes to avoid confusion
    scope.id = undefined;
    dispatch('change', { scope, actions });
  }

  function updateScopeId(e: Event) {
    const target = e.target as HTMLInputElement;
    scope.id = target.value;
    dispatch('change', { scope, actions });
  }
</script>

<div class="bg-surface-800 p-4 rounded-lg space-y-4 border border-surface-700">
  <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
    <!-- Scope Selection -->
    <div>
      <label class="label text-sm font-medium mb-1" for="scope-type">Scope</label>
      <select id="scope-type" class="select w-full" value={scope.type} on:change={updateScopeType}>
        {#each scopeOptions as opt}
          <option value={opt}>{opt}</option>
        {/each}
      </select>
    </div>

    <!-- Resource ID (Optional depending on scope) -->
    {#if scope.type !== 'System' && scope.type !== 'Tenant'} 
      <div>
        <label class="label text-sm font-medium mb-1" for="resource-id">Resource ID / Name</label>
        <input 
          type="text" 
          id="resource-id"
          class="input w-full" 
          placeholder={`e.g. "finance_db" for Namespace`}
          value={scope.id || ''} 
          on:input={updateScopeId}
        />
        <p class="text-xs text-surface-400 mt-1">Leave empty for all resources of this type</p>
      </div>
    {/if}
  </div>

  <!-- Actions Selection -->
  <div>
    <label class="label text-sm font-medium mb-2">Actions</label>
    <div class="flex flex-wrap gap-2">
      {#each actionOptions as action}
        <button
          type="button"
          class="chip {actions.includes(action) ? 'variant-filled-primary' : 'variant-soft-surface'}"
          on:click={() => toggleAction(action)}
        >
          {#if actions.includes(action)}
            <span>✓</span>
          {/if}
          <span>{action}</span>
        </button>
      {/each}
    </div>
  </div>
</div>
