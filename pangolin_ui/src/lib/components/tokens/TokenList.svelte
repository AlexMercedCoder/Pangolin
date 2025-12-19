<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import type { TokenInfo } from '$lib/api/tokens';
	import Button from '$lib/components/ui/Button.svelte';
	import DataTable from '$lib/components/ui/DataTable.svelte';

	export let tokens: TokenInfo[] = [];
	export let loading = false;

	const dispatch = createEventDispatcher();

	function formatDate(dateString: string): string {
		return new Date(dateString).toLocaleString();
	}

	function truncateId(id: string): string {
		return `${id.substring(0, 8)}...${id.substring(id.length - 8)}`;
	}

	function handleRevoke(token: TokenInfo) {
		dispatch('revoke', token);
	}

	const columns = [
		{ key: 'id', label: 'Token ID', width: '25%' },
		{ key: 'created_at', label: 'Created', width: '25%' },
		{ key: 'expires_at', label: 'Expires', width: '25%' },
		{ key: 'actions', label: 'Actions', width: '25%' }
	];
</script>

{#if loading}
	<div class="flex justify-center items-center py-8">
		<div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600"></div>
	</div>
{:else if tokens.length === 0}
	<div class="text-center py-8 text-gray-500 dark:text-gray-400">
		<p>No active tokens found.</p>
	</div>
{:else}
	<div class="overflow-x-auto">
		<table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
			<thead class="bg-gray-50 dark:bg-gray-800">
				<tr>
					{#each columns as column}
						<th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider" style="width: {column.width}">
							{column.label}
						</th>
					{/each}
				</tr>
			</thead>
			<tbody class="bg-white dark:bg-gray-900 divide-y divide-gray-200 dark:divide-gray-700">
				{#each tokens as token}
					<tr class="hover:bg-gray-50 dark:hover:bg-gray-800">
						<td class="px-6 py-4 whitespace-nowrap text-sm font-mono text-gray-900 dark:text-gray-100">
							{truncateId(token.id)}
						</td>
						<td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
							{formatDate(token.created_at)}
						</td>
						<td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
							{formatDate(token.expires_at)}
						</td>
						<td class="px-6 py-4 whitespace-nowrap text-sm">
							<Button
								variant="error"
								size="sm"
								on:click={() => handleRevoke(token)}
							>
								Revoke
							</Button>
						</td>
					</tr>
				{/each}
			</tbody>
		</table>
	</div>
{/if}
