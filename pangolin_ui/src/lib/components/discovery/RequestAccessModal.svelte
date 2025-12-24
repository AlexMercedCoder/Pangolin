<script lang="ts">
    import { businessMetadataApi } from '$lib/api/business_metadata';
    import { notifications } from '$lib/stores/notifications';
    
    export let open: boolean = false;
    export let assetId: string | null = null;
    export let assetName: string = 'Asset';
    
    let requestReason = '';
    let submitting = false;

    async function submitRequest() {
        if (!assetId) return;
        submitting = true;
        try {
            await businessMetadataApi.requestAccess(assetId, { reason: requestReason });
            notifications.success('Access request submitted');
            open = false;
            requestReason = '';
        } catch (e) {
            console.error(e);
            notifications.error('Failed to submit request');
        } finally {
            submitting = false;
        }
    }
    
    function cancel() {
        open = false;
        requestReason = '';
    }
</script>

{#if open}
<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
    <div class="bg-white dark:bg-gray-800 rounded-lg shadow-xl w-full max-w-lg p-6">
        <h3 class="text-xl font-bold text-gray-900 dark:text-white mb-4">Request Access: {assetName}</h3>
        
        <div class="mb-4">
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                Reason for Access
            </label>
            <textarea
                bind:value={requestReason}
                rows="4"
                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-gray-50 dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-primary-500 focus:outline-none"
                placeholder="Please describe why you need access to this dataset..."
            ></textarea>
        </div>
        
        <div class="flex justify-end gap-3">
            <button
                on:click={cancel}
                disabled={submitting}
                class="px-4 py-2 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg disabled:opacity-50"
            >
                Cancel
            </button>
            <button
                on:click={submitRequest}
                disabled={submitting || !requestReason.trim()}
                class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50 flex items-center gap-2"
            >
                {#if submitting}
                    <div class="animate-spin h-4 w-4 border-2 border-white border-t-transparent rounded-full"></div>
                {/if}
                Submit Request
            </button>
        </div>
    </div>
</div>
{/if}
