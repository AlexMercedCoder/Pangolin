<script lang="ts">
	import Button from '$lib/components/ui/Button.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import { notifications } from '$lib/stores/notifications';

	export let open = false;
	export let apiKey = '';
	export let keyName = '';

	async function copyApiKey() {
		try {
			await navigator.clipboard.writeText(apiKey);
			notifications.success('API key copied to clipboard');
		} catch (err) {
			notifications.error('Failed to copy API key');
		}
	}
</script>

<Modal bind:open title="API Key Generated" on:close={() => open = false}>
	<div class="space-y-4">
		<div class="bg-yellow-50 dark:bg-yellow-900/30 p-4 rounded-md border border-yellow-200 dark:border-yellow-800">
			<p class="text-sm text-yellow-800 dark:text-yellow-200 font-medium">
				Warning: This API key will only be shown once. Please copy it and store it securely.
			</p>
		</div>

		<div>
			<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
				API Key for <strong>{keyName}</strong>
			</label>
			<div class="relative">
				<textarea 
					class="w-full px-3 py-2 border rounded-md dark:bg-gray-800 dark:border-gray-700 dark:text-white font-mono text-sm h-24 pr-12 resize-none"
					readonly
					value={apiKey}
				></textarea>
				<button 
					class="absolute top-2 right-2 p-2 text-gray-500 hover:text-gray-700 dark:hover:text-gray-300 bg-white dark:bg-gray-800 rounded border border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
					on:click={copyApiKey}
					title="Copy to clipboard"
				>
					<span class="material-icons text-sm">content_copy</span>
				</button>
			</div>
		</div>
	</div>
	<div slot="footer">
		<Button on:click={() => open = false}>Close</Button>
	</div>
</Modal>
