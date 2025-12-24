<script lang="ts">
    import Modal from '$lib/components/ui/Modal.svelte';
    import Input from '$lib/components/ui/Input.svelte';
    import Select from '$lib/components/ui/Select.svelte';
    import Button from '$lib/components/ui/Button.svelte';
    import { assetsApi, type AssetType } from '$lib/api/assets';
    import { notifications } from '$lib/stores/notifications';

    export let open = false;
    export let catalogName: string;
    export let namespace: string;
    export let onSuccess: () => void = () => {};

    let loading = false;
    let name = '';
    let assetType: AssetType = 'DELTA_TABLE';
    let location = '';
    let properties: { key: string; value: string }[] = [];

    const assetTypeOptions = [
        { value: 'DELTA_TABLE', label: 'Delta Lake Table' },
        { value: 'HUDI_TABLE', label: 'Apache Hudi Table' },
        { value: 'PARQUET_TABLE', label: 'Parquet Table' },
        { value: 'CSV_TABLE', label: 'CSV Table' },
        { value: 'JSON_TABLE', label: 'JSON Table' },
        { value: 'VIEW', label: 'View' },
        { value: 'ML_MODEL', label: 'ML Model' },
    ];

    function addProperty() {
        properties = [...properties, { key: '', value: '' }];
    }

    function removeProperty(index: number) {
        properties = properties.filter((_, i) => i !== index);
    }

    async function handleSubmit() {
        console.log('RegisterAssetModal: handleSubmit called');
        console.log('Payload:', { catalogName, namespace, name, assetType, location });
        
        if (!name || !location) {
            console.log('Validation failed');
            notifications.error('Name and location are required');
            return;
        }

        // Convert properties array to Record<string, string>
        const propsRecord: Record<string, string> = {};
        properties.forEach(p => {
            if (p.key.trim()) {
                propsRecord[p.key.trim()] = p.value;
            }
        });

        loading = true;
        try {
            console.log('Calling assetsApi.register...');
            await assetsApi.register(catalogName, namespace, {
                name,
                kind: assetType,
                location,
                properties: propsRecord
            });
            console.log('Registration successful');
            notifications.success(`Asset "${name}" registered successfully`);
            open = false;
            resetForm();
            onSuccess();
        } catch (error: any) {
            console.error('Registration failed:', error);
            notifications.error(`Failed to register asset: ${error.message}`);
        } finally {
            loading = false;
        }
    }

    function resetForm() {
        name = '';
        assetType = 'DELTA_TABLE';
        location = '';
        properties = [];
    }

    $: if (!open) resetForm();
</script>

<Modal bind:open title="Register Generic Asset">
    <form on:submit|preventDefault={handleSubmit} class="space-y-4">
        <Input
            label="Asset Name"
            bind:value={name}
            placeholder="my_delta_table"
            required
        />

        <Select
            label="Asset Type"
            bind:value={assetType}
            options={assetTypeOptions}
            required
        />

        <Input
            label="Storage Location"
            bind:value={location}
            placeholder="s3://bucket/path/to/table or abfss://..."
            required
        />
        
        <div class="space-y-2">
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">Properties</label>
            <div class="space-y-2">
                {#each properties as prop, i}
                    <div class="flex gap-2">
                        <input
                            class="flex-1 rounded-md border-gray-300 dark:border-gray-600 shadow-sm focus:border-primary-500 focus:ring-primary-500 dark:bg-gray-700 dark:text-white sm:text-sm px-3 py-2"
                            placeholder="Key"
                            bind:value={prop.key}
                        />
                        <input
                            class="flex-1 rounded-md border-gray-300 dark:border-gray-600 shadow-sm focus:border-primary-500 focus:ring-primary-500 dark:bg-gray-700 dark:text-white sm:text-sm px-3 py-2"
                            placeholder="Value"
                            bind:value={prop.value}
                        />
                        <button
                            type="button"
                            class="text-red-500 hover:text-red-700 p-2"
                            on:click={() => removeProperty(i)}
                        >
                            <span class="material-icons text-sm">close</span>
                        </button>
                    </div>
                {/each}
                <button
                    type="button"
                    class="text-sm text-primary-600 hover:text-primary-700 font-medium flex items-center gap-1"
                    on:click={addProperty}
                >
                    <span class="material-icons text-sm">add</span> Add Property
                </button>
            </div>
        </div>

        <div class="p-4 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
            <p class="text-sm text-blue-800 dark:text-blue-200">
                <strong>💡 Note:</strong> This registers a non-Iceberg asset for discovery and metadata management. 
                The asset must already exist at the specified location.
            </p>
        </div>

        <div class="flex justify-end gap-3 pt-4">
            <Button variant="secondary" on:click={() => (open = false)} disabled={loading}>
                Cancel
            </Button>
            <Button type="submit" disabled={loading}>
                {loading ? 'Registering...' : 'Register Asset'}
            </Button>
        </div>
    </form>
</Modal>
