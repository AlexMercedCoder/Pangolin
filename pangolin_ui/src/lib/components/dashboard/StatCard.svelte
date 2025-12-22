<script lang="ts">
  export let label: string;
  export let value: number | string;
  export let icon: string;
  export let color: 'blue' | 'purple' | 'green' | 'yellow' | 'red' | 'gray' = 'blue';

  const colors = {
    blue: 'bg-blue-100 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400',
    purple: 'bg-purple-100 dark:bg-purple-900/30 text-purple-600 dark:text-purple-400',
    green: 'bg-green-100 dark:bg-green-900/30 text-green-600 dark:text-green-400',
    yellow: 'bg-yellow-100 dark:bg-yellow-900/30 text-yellow-600 dark:text-yellow-400',
    red: 'bg-red-100 dark:bg-red-900/30 text-red-600 dark:text-red-400',
    gray: 'bg-gray-100 dark:bg-gray-800 text-gray-600 dark:text-gray-400'
  };
</script>

<div class="bg-white dark:bg-gray-800 p-6 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700">
  <div class="flex items-center gap-4">
    <div class={`p-3 rounded-lg ${colors[color]}`}>
      <span class="text-2xl">{icon}</span>
    </div>
    <div>
      <p class="text-sm font-medium text-gray-500 dark:text-gray-400">{label}</p>
      {#if value === undefined || value === null}
        <div class="h-8 w-16 bg-gray-200 dark:bg-gray-700 rounded animate-pulse mt-1"></div>
      {:else}
        <p class="text-2xl font-semibold text-gray-900 dark:text-white">
          {typeof value === 'number' ? value.toLocaleString() : value}
        </p>
      {/if}
    </div>
  </div>
</div>
