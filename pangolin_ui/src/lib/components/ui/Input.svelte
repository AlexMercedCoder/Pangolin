<script lang="ts">
	export let type: 'text' | 'email' | 'password' | 'number' = 'text';
	export let label = '';
	export let placeholder = '';
	export let value = '';
	export let error = '';
	export let disabled = false;
	export let required = false;
	export let id = '';

	const inputId = id || `input-${Math.random().toString(36).substr(2, 9)}`;
</script>

<div class="w-full">
	{#if label}
		<label for={inputId} class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
			{label}
			{#if required}
				<span class="text-error-500">*</span>
			{/if}
		</label>
	{/if}
	<input
		{type}
		{placeholder}
		{disabled}
		{required}
		id={inputId}
		bind:value
		class="block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg shadow-sm 
			   focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-transparent
			   disabled:bg-gray-100 disabled:cursor-not-allowed
			   dark:bg-gray-800 dark:text-white
			   {error ? 'border-error-500 focus:ring-error-500' : ''}"
		on:input
		on:change
		on:blur
	/>
	{#if error}
		<p class="mt-1 text-sm text-error-600 dark:text-error-400">{error}</p>
	{/if}
</div>
