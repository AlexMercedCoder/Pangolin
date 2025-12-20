<script lang="ts">
	export let variant: 'info' | 'success' | 'warning' | 'error' | 'primary' | 'secondary' = 'info';
	
	const variants = {
		info: 'bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-300',
		success: 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300',
		warning: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-300',
		error: 'bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-300',
		primary: 'bg-primary-100 text-primary-800 dark:bg-primary-900/30 dark:text-primary-300',
		secondary: 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-300'
	};
</script>

<span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {variants[variant]}">
	<slot />
</span>
