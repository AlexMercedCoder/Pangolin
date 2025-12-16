<script lang="ts">
    import Modal from '$lib/components/ui/Modal.svelte';
    import Button from '$lib/components/ui/Button.svelte';
    import { createEventDispatcher } from 'svelte';
    import type { CreateTableRequest } from '$lib/api/iceberg';

    export let open = false;
    export let loading = false;

    let tableName = '';
    let fields: { name: string; type: string; required: boolean }[] = [
        { name: 'id', type: 'int', required: true }
    ];

    const dispatch = createEventDispatcher();
    const PRIMITIVE_TYPES = ['string', 'int', 'long', 'float', 'double', 'boolean', 'timestamp', 'date'];

    function addField() {
        fields = [...fields, { name: '', type: 'string', required: false }];
    }

    function removeField(index: number) {
        fields = fields.filter((_, i) => i !== index);
    }

    function handleSubmit() {
        if (!tableName.trim()) return;
        if (fields.length === 0) return;

        // Construct request
        const request: CreateTableRequest = {
            name: tableName.trim(),
            schema: {
                type: 'struct',
                fields: fields.map((f, i) => ({
                    id: i + 1, // Auto-assign IDs
                    name: f.name,
                    required: f.required,
                    type: f.type
                }))
            }
        };

        dispatch('create', request);
        resetForm();
    }

    function resetForm() {
        tableName = '';
        fields = [{ name: 'id', type: 'int', required: true }];
    }

    function handleCancel() {
        open = false;
        resetForm();
    }
</script>

<Modal bind:open title="Create Table">
    <div class="space-y-6 max-h-[70vh] overflow-y-auto pr-2 custom-scrollbar">
        <!-- Table Name -->
        <div>
            <label for="table-name" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                Table Name
            </label>
            <input
                id="table-name"
                type="text"
                bind:value={tableName}
                placeholder="e.g. employees"
                class="mt-1 block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-primary-500 focus:border-primary-500 bg-white dark:bg-gray-700 dark:text-white sm:text-sm"
            />
        </div>

        <!-- Schema Builder -->
        <div>
            <div class="flex items-center justify-between mb-2">
                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                    Schema Definition
                </label>
                <Button size="sm" variant="secondary" on:click={addField}>+ Add Column</Button>
            </div>
            
            <div class="space-y-3 bg-gray-50 dark:bg-gray-800/50 p-3 rounded-lg border border-gray-200 dark:border-gray-700">
                {#each fields as field, i}
                    <div class="flex items-start gap-2">
                        <div class="flex-1">
                            <input
                                type="text"
                                placeholder="Column Name"
                                bind:value={field.name}
                                class="w-full px-2 py-1.5 text-sm border border-gray-300 dark:border-gray-600 rounded focus:ring-primary-500 focus:border-primary-500 bg-white dark:bg-gray-700 dark:text-white"
                            />
                        </div>
                        <div class="w-32">
                            <select
                                bind:value={field.type}
                                class="w-full px-2 py-1.5 text-sm border border-gray-300 dark:border-gray-600 rounded focus:ring-primary-500 focus:border-primary-500 bg-white dark:bg-gray-700 dark:text-white"
                            >
                                {#each PRIMITIVE_TYPES as t}
                                    <option value={t}>{t}</option>
                                {/each}
                            </select>
                        </div>
                        <div class="flex items-center h-8">
                            <input
                                type="checkbox"
                                bind:checked={field.required}
                                title="Required"
                                class="h-4 w-4 text-primary-600 focus:ring-primary-500 border-gray-300 rounded"
                            />
                        </div>
                        <button
                            on:click={() => removeField(i)}
                            class="p-1.5 text-gray-400 hover:text-red-500 transition-colors"
                            title="Remove Column"
                        >
                            ✕
                        </button>
                    </div>
                {/each}
                {#if fields.length === 0}
                    <div class="text-center text-sm text-gray-500 italic py-2">
                        No columns added.
                    </div>
                {/if}
            </div>
        </div>

        <!-- Actions -->
        <div class="flex justify-end gap-3 pt-2">
            <Button variant="secondary" on:click={handleCancel} disabled={loading}>
                Cancel
            </Button>
            <Button on:click={handleSubmit} disabled={loading || !tableName.trim() || fields.length === 0}>
                {loading ? 'Creating...' : 'Create Table'}
            </Button>
        </div>
    </div>
</Modal>
