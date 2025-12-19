<script lang="ts">
	export let type: 'text' | 'email' | 'password' | 'number' = 'text';
	export let label = '';
	export let placeholder = '';
	export let value = '';
	export let error = '';
	export let disabled = false;
	export let required = false;
	export let id = '';
	export let helpText = '';
	export let tooltip = '';

	const inputId = id || `input-${Math.random().toString(36).substr(2, 9)}`;
</script>

<div class="w-full">
	{#if label}
		<div class="flex items-center gap-1.5 mb-1">
			<label for={inputId} class="block text-sm font-medium text-gray-700 dark:text-gray-300">
				{label}
				{#if required}
					<span class="text-error-500">*</span>
				{/if}
			</label>
			{#if tooltip}
				<div class="group relative flex items-center">
					<span class="material-icons text-xs text-gray-400 cursor-help hover:text-primary-500 transition-colors">help</span>
					<div class="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 w-48 p-2 bg-gray-900 text-white text-xs rounded shadow-lg 
								opacity-0 pointer-events-none group-hover:opacity-100 transition-opacity z-10 text-center font-normal">
						{tooltip}
						<div class="absolute top-full left-1/2 -translate-x-1/2 border-8 border-transparent border-t-gray-900"></div>
					</div>
				</div>
			{/if}
		</div>
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
	{:else if helpText}
		<p class="mt-1 text-sm text-gray-500 dark:text-gray-400">{helpText}</p>
	{/if}
</div>
