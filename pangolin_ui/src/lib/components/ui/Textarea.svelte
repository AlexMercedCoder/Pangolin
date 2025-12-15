<script lang="ts">
  export let label: string = '';
  export let value: string = '';
  export let error: string = '';
  export let helpText: string = '';
  export let required: boolean = false;
  export let disabled: boolean = false;
  export let rows: number = 4;
  export let maxlength: number | undefined = undefined;
  export let placeholder: string = '';

  $: characterCount = value?.length || 0;
  $: showCounter = maxlength !== undefined;
</script>

<div class="mb-4">
  {#if label}
    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
      {label}
      {#if required}
        <span class="text-red-500">*</span>
      {/if}
    </label>
  {/if}
  
  <textarea
    bind:value
    {disabled}
    {required}
    {rows}
    {maxlength}
    {placeholder}
    class="w-full px-3 py-2 border rounded-lg shadow-sm transition-colors resize-y
      {error 
        ? 'border-red-500 focus:border-red-500 focus:ring-red-500' 
        : 'border-gray-300 dark:border-gray-600 focus:border-primary-500 focus:ring-primary-500'}
      bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100
      disabled:bg-gray-100 dark:disabled:bg-gray-700 disabled:cursor-not-allowed
      focus:outline-none focus:ring-2 focus:ring-opacity-50"></textarea>
  
  <div class="flex justify-between items-center mt-1">
    <div class="flex-1">
      {#if helpText && !error}
        <p class="text-sm text-gray-500 dark:text-gray-400">{helpText}</p>
      {/if}
      
      {#if error}
        <p class="text-sm text-red-500">{error}</p>
      {/if}
    </div>
    
    {#if showCounter}
      <p class="text-sm text-gray-500 dark:text-gray-400 ml-2">
        {characterCount}{#if maxlength}/{maxlength}{/if}
      </p>
    {/if}
  </div>
</div>
