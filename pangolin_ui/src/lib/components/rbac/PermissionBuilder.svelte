<script context="module" lang="ts">
  // Types based on backend Permission struct
  export type ScopeType = 'System' | 'Tenant' | 'Catalog' | 'Namespace' | 'Table' | 'View';
  export type ActionType = 'READ' | 'WRITE' | 'CREATE' | 'DELETE' | 'ALL' | 'MANAGE_DISCOVERY' | 'LIST';
</script>

<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  
  // Re-export or use directly (in Svelte <script> they are available)
  // We don't need to re-define types but we need variables using them.


  import { isRoot } from '$lib/stores/auth';

  export let scope: { type: ScopeType; id?: string } = { type: 'Tenant' };
  export let actions: ActionType[] = [];
  
  const dispatch = createEventDispatcher();

  const allScopeOptions: ScopeType[] = ['System', 'Tenant', 'Catalog', 'Namespace', 'Table', 'View'];
  $: scopeOptions = $isRoot ? allScopeOptions : allScopeOptions.filter(s => s !== 'System');
  const actionOptions: ActionType[] = ['READ', 'WRITE', 'CREATE', 'DELETE', 'LIST', 'ALL', 'MANAGE_DISCOVERY'];

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
      <select id="scope-type" class="select w-full text-gray-900 dark:text-white bg-white dark:bg-gray-800" value={scope.type} on:change={updateScopeType}>
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
          class="input w-full text-gray-900 dark:text-white bg-white dark:bg-gray-800" 
          placeholder={`e.g. "finance_db" for Namespace`}
          value={scope.id || ''} 
          on:input={updateScopeId}
        />
        <p class="text-xs text-surface-400 mt-1">Leave empty for all resources of this type</p>
      </div>
      
      <!-- Contextual Guidance -->
      <div class="col-span-1 md:col-span-2 bg-surface-700/50 p-3 rounded text-sm text-surface-200">
          <span class="font-bold text-primary-400">💡 Guidance:</span>
          {#if scope.type === 'Catalog'}
              Enter the exact <strong>Catalog Name</strong>. It will be resolved to its ID automatically.
          {:else if scope.type === 'Namespace'}
              Use format <code>catalogName.namespaceName</code> (e.g., <code>lakehouse_cat.sales</code>). Nested namespaces use dots (e.g., <code>lakehouse_cat.sales.regional</code>).
          {:else if scope.type === 'Table' || scope.type === 'View'}
               Use format <code>catalogName.namespaceName.tableName</code>.
          {:else}
               Enter the Resource ID or Name.
          {/if}
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
