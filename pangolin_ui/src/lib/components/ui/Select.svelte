<script lang="ts">
	export let options: Array<{ value: string; label: string }> = [];
	export let value: string = '';
	export let placeholder = 'Select an option';
	export let disabled = false;
	export let error = '';
	export let label = '';
	export let required = false;
</script>

<div class="space-y-1">
	{#if label}
		<label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
			{label}
			{#if required}
				<span class="text-error-600">*</span>
			{/if}
		</label>
	{/if}
	<select
		bind:value
		{disabled}
		class="w-full px-4 py-2 border rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white
			{error
			? 'border-error-500 focus:ring-error-500'
			: 'border-gray-300 dark:border-gray-600 focus:ring-primary-500'}
			focus:ring-2 focus:border-transparent
			disabled:opacity-50 disabled:cursor-not-allowed
			transition-colors"
	>
		<option value="" disabled selected>{placeholder}</option>
		{#each options as option}
			<option value={option.value}>{option.label}</option>
		{/each}
	</select>
	{#if error}
		<p class="text-sm text-error-600">{error}</p>
	{/if}
</div>
