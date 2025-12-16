<script lang="ts">
    import Modal from '$lib/components/ui/Modal.svelte';
    import Button from '$lib/components/ui/Button.svelte';
    import { createEventDispatcher } from 'svelte';

    export let open = false;
    export let loading = false;

    let namespaceName = '';
    const dispatch = createEventDispatcher();

    function handleSubmit() {
        if (!namespaceName.trim()) return;
        dispatch('create', { name: namespaceName.trim() });
        namespaceName = ''; // Reset
    }

    function handleCancel() {
        open = false;
        namespaceName = '';
    }
</script>

<Modal bind:open title="Create Namespace">
    <div class="space-y-4">
        <div>
            <label for="namespace-name" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                Namespace Name
            </label>
            <input
                id="namespace-name"
                type="text"
                bind:value={namespaceName}
                placeholder="e.g. sales_data"
                class="mt-1 block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-primary-500 focus:border-primary-500 bg-white dark:bg-gray-700 dark:text-white sm:text-sm"
                on:keydown={(e) => e.key === 'Enter' && handleSubmit()}
            />
            <p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
                Use dots for nested namespaces (e.g. <code>dept.team</code>) if supported, or create sequentially.
            </p>
        </div>

        <div class="flex justify-end gap-3 pt-4">
            <Button variant="secondary" on:click={handleCancel} disabled={loading}>
                Cancel
            </Button>
            <Button on:click={handleSubmit} disabled={loading || !namespaceName.trim()}>
                {loading ? 'Creating...' : 'Create Namespace'}
            </Button>
        </div>
    </div>
</Modal>
