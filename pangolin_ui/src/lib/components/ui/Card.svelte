<script lang="ts">
	export let title = '';
	export let padding = true;
	export let elevation: 1 | 2 | 3 | 4 = 2;
</script>

<div class="bg-white dark:bg-gray-800 rounded-lg elevation-{elevation} {padding ? 'p-6' : ''}">
	{#if title}
		<h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">{title}</h3>
	{/if}
	<slot />
</div>
