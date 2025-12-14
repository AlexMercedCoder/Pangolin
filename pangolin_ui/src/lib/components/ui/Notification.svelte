<script lang="ts">
	import { notifications } from '$lib/stores/notifications';
	import { fade, fly } from 'svelte/transition';

	const iconMap = {
		success: '✓',
		error: '✕',
		warning: '⚠',
		info: 'ℹ',
	};

	const colorMap = {
		success: 'bg-success-50 dark:bg-success-900/20 border-success-500 text-success-800 dark:text-success-200',
		error: 'bg-error-50 dark:bg-error-900/20 border-error-500 text-error-800 dark:text-error-200',
		warning: 'bg-warning-50 dark:bg-warning-900/20 border-warning-500 text-warning-800 dark:text-warning-200',
		info: 'bg-blue-50 dark:bg-blue-900/20 border-blue-500 text-blue-800 dark:text-blue-200',
	};
</script>

<div class="fixed top-4 right-4 z-50 space-y-2 max-w-md">
	{#each $notifications as notification (notification.id)}
		<div
			transition:fly={{ x: 300, duration: 300 }}
			class="flex items-start gap-3 p-4 rounded-lg border-l-4 shadow-lg {colorMap[notification.type]}"
		>
			<span class="text-xl font-bold">{iconMap[notification.type]}</span>
			<p class="flex-1 text-sm font-medium">{notification.message}</p>
			<button
				on:click={() => notifications.remove(notification.id)}
				class="text-current opacity-70 hover:opacity-100 transition-opacity"
			>
				✕
			</button>
		</div>
	{/each}
</div>
