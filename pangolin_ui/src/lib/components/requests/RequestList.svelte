<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import type { AccessRequest } from '$lib/api/access_requests';
	import Button from '$lib/components/ui/Button.svelte';
	import Card from '$lib/components/ui/Card.svelte';

	export let requests: AccessRequest[] = [];
	export let loading = false;
	export let filter: 'all' | 'pending' | 'approved' | 'rejected' = 'all';

	const dispatch = createEventDispatcher();

	$: filteredRequests = filter === 'all' 
		? requests 
		: requests.filter(r => r.status === filter);

	function formatDate(dateString: string): string {
		return new Date(dateString).toLocaleString();
	}

	function getStatusColor(status: string): string {
		switch (status) {
			case 'pending': return 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200';
			case 'approved': return 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200';
			case 'rejected': return 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200';
			default: return 'bg-gray-100 text-gray-800 dark:bg-gray-900 dark:text-gray-200';
		}
	}

	function handleApprove(request: AccessRequest) {
		dispatch('approve', request);
	}

	function handleReject(request: AccessRequest) {
		dispatch('reject', request);
	}

	function handleView(request: AccessRequest) {
		dispatch('view', request);
	}
</script>

{#if loading}
	<div class="flex justify-center items-center py-8">
		<div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600"></div>
	</div>
{:else if filteredRequests.length === 0}
	<div class="text-center py-8 text-gray-500 dark:text-gray-400">
		<p>No {filter === 'all' ? '' : filter} access requests found.</p>
	</div>
{:else}
	<div class="space-y-4">
		{#each filteredRequests as request}
			<Card>
				<div class="p-6">
					<div class="flex justify-between items-start mb-4">
						<div class="flex-1">
							<h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">
								{request.asset_fqn || request.asset_name || request.asset_id}
							</h3>
							<p class="text-sm text-gray-500 dark:text-gray-400 mt-1">
								Requested by: {request.user_name || request.user_id}
							</p>
						</div>
						<span class="px-3 py-1 rounded-full text-xs font-medium {getStatusColor(request.status)}">
							{request.status.toUpperCase()}
						</span>
					</div>

					<div class="mb-4">
						<p class="text-sm text-gray-700 dark:text-gray-300">
							<strong>Reason:</strong> {request.reason}
						</p>
						<p class="text-xs text-gray-500 dark:text-gray-400 mt-2">
							Requested: {formatDate(request.requested_at)}
						</p>
						{#if request.reviewed_at}
							<p class="text-xs text-gray-500 dark:text-gray-400">
								Reviewed: {formatDate(request.reviewed_at)}
								{#if request.reviewed_by}by {request.reviewed_by}{/if}
							</p>
						{/if}
					</div>

					<div class="flex gap-2">
						{#if request.status === 'pending'}
							<Button
								variant="success"
								size="sm"
								on:click={() => handleApprove(request)}
							>
								Approve
							</Button>
							<Button
								variant="error"
								size="sm"
								on:click={() => handleReject(request)}
							>
								Reject
							</Button>
						{/if}
						<Button
							variant="ghost"
							size="sm"
							on:click={() => handleView(request)}
						>
							View Details
						</Button>
					</div>
				</div>
			</Card>
		{/each}
	</div>
{/if}
