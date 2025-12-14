<script lang="ts">
	export let variant: 'primary' | 'secondary' | 'success' | 'error' | 'ghost' = 'primary';
	export let size: 'sm' | 'md' | 'lg' = 'md';
	export let type: 'button' | 'submit' | 'reset' = 'button';
	export let disabled = false;
	export let loading = false;
	export let fullWidth = false;

	const baseClasses = 'inline-flex items-center justify-center font-medium rounded-lg transition-all focus:outline-none focus:ring-2 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed';
	
	const variantClasses = {
		primary: 'bg-primary-600 text-white hover:bg-primary-700 focus:ring-primary-500',
		secondary: 'bg-secondary-600 text-white hover:bg-secondary-700 focus:ring-secondary-500',
		success: 'bg-success-600 text-white hover:bg-success-700 focus:ring-success-500',
		error: 'bg-error-600 text-white hover:bg-error-700 focus:ring-error-500',
		ghost: 'bg-transparent text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800 focus:ring-gray-500',
	};

	const sizeClasses = {
		sm: 'px-3 py-1.5 text-sm',
		md: 'px-4 py-2 text-base',
		lg: 'px-6 py-3 text-lg',
	};

	$: classes = `${baseClasses} ${variantClasses[variant]} ${sizeClasses[size]} ${fullWidth ? 'w-full' : ''}`;
</script>

<button
	{type}
	{disabled}
	class={classes}
	on:click
>
	{#if loading}
		<svg class="animate-spin -ml-1 mr-2 h-4 w-4" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
			<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
			<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
		</svg>
	{/if}
	<slot />
</button>
