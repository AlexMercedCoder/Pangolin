<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import Button from './Button.svelte';

	export let open = false;
	export let title = '';
	export let size: 'sm' | 'md' | 'lg' = 'md';

	const dispatch = createEventDispatcher();

	const sizeClasses = {
		sm: 'max-w-md',
		md: 'max-w-lg',
		lg: 'max-w-2xl',
	};

	function close() {
		open = false;
		dispatch('close');
	}

	function handleBackdropClick(e: MouseEvent) {
		if (e.target === e.currentTarget) {
			close();
		}
	}
</script>

{#if open}
	<div
		class="fixed inset-0 z-50 overflow-y-auto"
		on:click={handleBackdropClick}
		on:keydown={(e) => e.key === 'Escape' && close()}
		role="dialog"
		aria-modal="true"
	>
		<div class="flex items-center justify-center min-h-screen px-4 pt-4 pb-20 text-center sm:block sm:p-0">
			<!-- Backdrop -->
			<div class="fixed inset-0 bg-gray-500 bg-opacity-75 dark:bg-gray-900 dark:bg-opacity-75 transition-opacity"></div>

			<!-- Modal panel -->
			<div class="inline-block align-bottom bg-white dark:bg-gray-800 rounded-lg text-left overflow-hidden shadow-xl transform transition-all sm:my-8 sm:align-middle w-full {sizeClasses[size]}">
				<!-- Header -->
				{#if title}
					<div class="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
						<h3 class="text-lg font-semibold text-gray-900 dark:text-white">{title}</h3>
					</div>
				{/if}

				<!-- Content -->
				<div class="px-6 py-4">
					<slot />
				</div>

				<!-- Footer -->
				<div class="px-6 py-4 bg-gray-50 dark:bg-gray-900 flex justify-end gap-3">
					<slot name="footer">
						<Button variant="ghost" on:click={close}>Cancel</Button>
					</slot>
				</div>
			</div>
		</div>
	</div>
{/if}
