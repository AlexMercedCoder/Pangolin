<script lang="ts">
	import { onMount } from 'svelte';
	import { businessMetadataApi, type AccessRequest } from '$lib/api/business_metadata';
    import { notifications } from '$lib/stores/notifications';
    import { authStore } from '$lib/stores/auth';

    let requests: AccessRequest[] = [];
    let loading = true;
    let selectedStatus: 'Pending' | 'Approved' | 'Rejected' | 'All' = 'Pending';
    
    // Modal state for Approve/Reject
    let showReviewModal = false;
    let selectedRequest: AccessRequest | null = null;
    let reviewAction: 'Approved' | 'Rejected' = 'Approved';
    let reviewComment = '';

	onMount(async () => {
        loadRequests();
	});

    async function loadRequests() {
        loading = true;
        try {
            requests = await businessMetadataApi.listRequests();
        } catch (e) {
            console.error(e);
            notifications.error('Failed to load access requests');
        } finally {
            loading = false;
            // TODO: In a real app we might fetch user names and asset names here
            // because `AccessRequest` only has IDs.
            // For now, we display IDs.
        }
    }

    $: filteredRequests = selectedStatus === 'All' 
        ? requests 
        : requests.filter(r => r.status === selectedStatus);

    function openReviewModal(request: AccessRequest, action: 'Approved' | 'Rejected') {
        selectedRequest = request;
        reviewAction = action;
        reviewComment = '';
        showReviewModal = true;
    }

    async function submitReview() {
        if (!selectedRequest) return;
        try {
            await businessMetadataApi.updateRequestStatus(selectedRequest.id, {
                status: reviewAction,
                comment: reviewComment
            });
            notifications.success(`Request ${reviewAction.toLowerCase()}`);
            showReviewModal = false;
            loadRequests();
        } catch (e) {
            console.error(e);
            notifications.error('Failed to update request');
        }
    }
</script>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<h1 class="text-2xl font-bold text-gray-900 dark:text-white">Access Requests</h1>
        <button 
            on:click={loadRequests} 
            class="p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-500"
            title="Refresh"
        >
            🔄
        </button>
	</div>

    <!-- Filters -->
    <div class="flex gap-2 border-b border-gray-200 dark:border-gray-700 pb-4">
        {#each ['Pending', 'Approved', 'Rejected', 'All'] as status}
            <button
                on:click={() => selectedStatus = status as any}
                class="px-4 py-2 text-sm font-medium rounded-lg transition-colors {selectedStatus === status ? 'bg-primary-100 text-primary-700 dark:bg-primary-900 dark:text-primary-300' : 'text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800'}"
            >
                {status}
            </button>
        {/each}
    </div>

    <!-- List -->
    {#if loading}
        <div class="flex justify-center py-12">
            <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600"></div>
        </div>
    {:else if filteredRequests.length === 0}
         <div class="text-center py-12 text-gray-500 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
            No {selectedStatus === 'All' ? '' : selectedStatus.toLowerCase()} requests found.
        </div>
    {:else}
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 overflow-hidden">
            <table class="w-full text-left">
                <thead class="bg-gray-50 dark:bg-gray-900/50 text-xs uppercase text-gray-500 dark:text-gray-400">
                    <tr>
                        <th class="px-6 py-3">Timestamp</th>
                        <th class="px-6 py-3">User ID</th>
                        <th class="px-6 py-3">Asset ID</th>
                        <th class="px-6 py-3">Status</th>
                        <th class="px-6 py-3">Reason</th>
                        <th class="px-6 py-3 text-right">Actions</th>
                    </tr>
                </thead>
                <tbody class="divide-y divide-gray-200 dark:divide-gray-700">
                    {#each filteredRequests as req}
                        <tr class="hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors">
                            <td class="px-6 py-4 text-sm text-gray-900 dark:text-white">
                                {new Date(req.requested_at).toLocaleString()}
                            </td>
                            <td class="px-6 py-4 text-sm text-gray-500 font-mono">
                                {req.user_id}
                            </td>
                            <td class="px-6 py-4 text-sm text-gray-500 font-mono">
                                {req.asset_id}
                            </td>
                            <td class="px-6 py-4">
                                <span class="px-2 py-1 text-xs font-medium rounded-full 
                                    {req.status === 'Approved' ? 'bg-green-100 text-green-800' : 
                                     req.status === 'Rejected' ? 'bg-error-100 text-error-800' : 
                                     'bg-yellow-100 text-yellow-800'}">
                                    {req.status}
                                </span>
                            </td>
                            <td class="px-6 py-4 text-sm text-gray-600 dark:text-gray-300 max-w-xs truncate" title={req.reason}>
                                {req.reason || '-'}
                            </td>
                            <td class="px-6 py-4 text-right space-x-2">
                                {#if req.status === 'Pending'}
                                    <button 
                                        on:click={() => openReviewModal(req, 'Approved')}
                                        class="text-xs font-medium text-green-600 hover:text-green-700 bg-green-50 hover:bg-green-100 px-3 py-1 rounded"
                                    >
                                        Approve
                                    </button>
                                    <button 
                                        on:click={() => openReviewModal(req, 'Rejected')}
                                        class="text-xs font-medium text-error-600 hover:text-error-700 bg-error-50 hover:bg-error-100 px-3 py-1 rounded"
                                    >
                                        Reject
                                    </button>
                                {:else}
                                    <span class="text-xs text-gray-400">
                                        Reviewed by {req.reviewed_by || 'System'}
                                    </span>
                                {/if}
                            </td>
                        </tr>
                    {/each}
                </tbody>
            </table>
        </div>
    {/if}
</div>

<!-- Review Modal -->
{#if showReviewModal && selectedRequest}
<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
    <div class="bg-white dark:bg-gray-800 rounded-lg shadow-xl w-full max-w-lg p-6">
        <h3 class="text-xl font-bold text-gray-900 dark:text-white mb-4">
            {reviewAction} Request
        </h3>
        
        <p class="mb-4 text-gray-600 dark:text-gray-300">
            You are about to <strong>{reviewAction.toLowerCase()}</strong> the request for asset 
            <code class="text-xs bg-gray-100 dark:bg-gray-700 px-1 rounded">{selectedRequest.asset_id}</code>
            from user <code class="text-xs bg-gray-100 dark:bg-gray-700 px-1 rounded">{selectedRequest.user_id}</code>.
        </p>

        <div class="mb-4">
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                Comment (Optional)
            </label>
            <textarea
                bind:value={reviewComment}
                rows="3"
                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-gray-50 dark:bg-gray-700 text-gray-900 dark:text-white"
                placeholder="Add a note to the user..."
            ></textarea>
        </div>
        
        <div class="flex justify-end gap-3">
            <button
                on:click={() => showReviewModal = false}
                class="px-4 py-2 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg"
            >
                Cancel
            </button>
            <button
                on:click={submitReview}
                class="px-4 py-2 text-white rounded-lg {reviewAction === 'Approved' ? 'bg-green-600 hover:bg-green-700' : 'bg-error-600 hover:bg-error-700'}"
            >
                Confirm {reviewAction}
            </button>
        </div>
    </div>
</div>
{/if}
