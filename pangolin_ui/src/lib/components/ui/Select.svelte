<script lang="ts">
	export let options: Array<{ value: string; label: string }> = [];
	export let value: string = '';
	export let placeholder = 'Select an option';
	export let disabled = false;
	export let error = '';
	export let helpText = '';
	export let label = '';
	export let id = '';
	export let required = false;
	export let tooltip = '';

	const selectId = id || `select-${Math.random().toString(36).substr(2, 9)}`;
</script>

<div class="space-y-1">
	{#if label}
		<div class="flex items-center gap-1.5 mb-1">
			<label for={selectId} class="block text-sm font-medium text-gray-700 dark:text-gray-300">
				{label}
				{#if required}
					<span class="text-error-600">*</span>
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
	<select
		id={selectId}
		bind:value
		{disabled}
		class="w-full px-4 py-2 border rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white
			{error
			? 'border-error-500 focus:ring-error-500'
			: 'border-gray-300 dark:border-gray-600 focus:ring-primary-500'}
			focus:ring-2 focus:border-transparent
			disabled:opacity-50 disabled:cursor-not-allowed
			transition-colors"
		on:change
	>
		<option value="" disabled selected>{placeholder}</option>
		{#each options as option}
			<option value={option.value}>{option.label}</option>
		{/each}
	</select>
	{#if helpText}
		<p class="mt-1 text-sm text-gray-500 dark:text-gray-400">{helpText}</p>
	{/if}
	{#if error}
		<p class="text-sm text-error-600">{error}</p>
	{/if}
</div>
