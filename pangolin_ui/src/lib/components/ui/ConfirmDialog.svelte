<script lang="ts">
	import Modal from './Modal.svelte';

	export let open = false;
	export let title = 'Confirm Action';
	export let message = 'Are you sure you want to proceed?';
	export let confirmText = 'Confirm';
	export let cancelText = 'Cancel';
	export let variant: 'danger' | 'warning' | 'info' = 'warning';
	export let onConfirm: () => void = () => {};

	export let onCancel: () => void = () => {};
	export let loading = false;

	const variantStyles = {
		danger: 'bg-error-600 hover:bg-error-700',
		warning: 'bg-warning-600 hover:bg-warning-700',
		info: 'bg-primary-600 hover:bg-primary-700',
	};

	const iconMap = {
		danger: '⚠️',
		warning: '⚠️',
		info: 'ℹ️',
	};

	function handleConfirm() {
		onConfirm();
		open = false;
	}

	function handleCancel() {
		onCancel();
		open = false;
	}
</script>

<Modal bind:open>
	<div class="space-y-4">
		<div class="flex items-start gap-4">
			<span class="text-4xl">{iconMap[variant]}</span>
			<div class="flex-1">
				<h3 class="text-lg font-semibold text-gray-900 dark:text-white">
					{title}
				</h3>
				<p class="mt-2 text-sm text-gray-600 dark:text-gray-400">
					{message}
				</p>
			</div>
		</div>

		<div class="flex items-center gap-3 justify-end pt-4 border-t border-gray-200 dark:border-gray-700">
			<button
				on:click={handleCancel}
				class="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
			>
				{cancelText}
			</button>
			<button
				on:click={handleConfirm}
				class="px-4 py-2 text-sm font-medium text-white rounded-lg {variantStyles[variant]} transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
				disabled={loading}
			>
				{#if loading}
					<svg class="animate-spin -ml-1 mr-2 h-4 w-4 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
						<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
						<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
					</svg>
					Processing...
				{:else}
					{confirmText}
				{/if}
			</button>
		</div>
	</div>
</Modal>
