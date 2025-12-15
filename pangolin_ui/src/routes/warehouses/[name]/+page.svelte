<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { warehousesApi } from '$lib/api/warehouses';
  import { notifications } from '$lib/stores/notifications';
  import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
  import type { Warehouse } from '$lib/api/warehouses';

  let warehouse: Warehouse | null = null;
  let loading = true;
  let showDeleteDialog = false;
  let deleting = false;

  const warehouseName = $page.params.name ?? '';

  onMount(async () => {
    if (warehouseName) {
      await loadWarehouse();
    } else {
      goto('/warehouses');
    }
  });

  async function loadWarehouse() {
    try {
      loading = true;
      if (!warehouseName) return;
      warehouse = await warehousesApi.get(warehouseName);
    } catch (error: any) {
      notifications.error(`Failed to load warehouse: ${error.message}`);
      goto('/warehouses');
    } finally {
      loading = false;
    }
  }

  async function handleDelete() {
    try {
      deleting = true;
      if (!warehouseName) return;
      await warehousesApi.delete(warehouseName);
      notifications.success(`Warehouse "${warehouseName}" deleted successfully`);
      goto('/warehouses');
    } catch (error: any) {
      notifications.error(`Failed to delete warehouse: ${error.message}`);
    } finally {
      deleting = false;
      showDeleteDialog = false;
    }
  }

  function getStorageType(config: any): string {
    return config.type?.toUpperCase() || 'Unknown';
  }

  function getAuthMethod(useSts: boolean): string {
    return useSts ? 'IAM Role / OAuth' : 'Static Credentials';
  }
</script>

<div class="container mx-auto px-4 py-8">
  {#if loading}
    <div class="flex items-center justify-center py-12">
      <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
    </div>
  {:else if warehouse}
    <!-- Header -->
    <div class="mb-6">
      <div class="flex items-center justify-between">
        <div>
          <button
            on:click={() => goto('/warehouses')}
            class="text-gray-600 hover:text-gray-900 mb-2 flex items-center gap-2"
          >
            ← Back to Warehouses
          </button>
          <h1 class="text-3xl font-bold text-gray-900">{warehouse.name}</h1>
        </div>
        <div class="flex gap-3">
          <button
            on:click={() => showDeleteDialog = true}
            class="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors"
          >
            Delete Warehouse
          </button>
        </div>
      </div>
    </div>

    <!-- Warehouse Details -->
    <div class="bg-white rounded-lg shadow-md p-6 mb-6">
      <h2 class="text-xl font-semibold mb-4">Warehouse Configuration</h2>
      
      <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
        <!-- Basic Info -->
        <div>
          <h3 class="text-sm font-medium text-gray-500 mb-3">Basic Information</h3>
          <dl class="space-y-3">
            <div>
              <dt class="text-sm font-medium text-gray-700">Name</dt>
              <dd class="mt-1 text-sm text-gray-900">{warehouse.name}</dd>
            </div>
            <div>
              <dt class="text-sm font-medium text-gray-700">Storage Type</dt>
              <dd class="mt-1">
                <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
                  {getStorageType(warehouse.storage_config)}
                </span>
              </dd>
            </div>
            <div>
              <dt class="text-sm font-medium text-gray-700">Authentication Method</dt>
              <dd class="mt-1">
                <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {warehouse.use_sts ? 'bg-green-100 text-green-800' : 'bg-gray-100 text-gray-800'}">
                  {getAuthMethod(warehouse.use_sts)}
                </span>
              </dd>
            </div>
          </dl>
        </div>

        <!-- Storage Configuration -->
        <div>
          <h3 class="text-sm font-medium text-gray-500 mb-3">Storage Configuration</h3>
          <dl class="space-y-3">
            {#if warehouse.storage_config.bucket}
              <div>
                <dt class="text-sm font-medium text-gray-700">Bucket/Container</dt>
                <dd class="mt-1 text-sm text-gray-900 font-mono">{warehouse.storage_config.bucket}</dd>
              </div>
            {/if}
            {#if warehouse.storage_config.region}
              <div>
                <dt class="text-sm font-medium text-gray-700">Region</dt>
                <dd class="mt-1 text-sm text-gray-900">{warehouse.storage_config.region}</dd>
              </div>
            {/if}
            {#if warehouse.storage_config.endpoint}
              <div>
                <dt class="text-sm font-medium text-gray-700">Custom Endpoint</dt>
                <dd class="mt-1 text-sm text-gray-900 font-mono">{warehouse.storage_config.endpoint}</dd>
              </div>
            {/if}
            {#if warehouse.storage_config.account_name}
              <div>
                <dt class="text-sm font-medium text-gray-700">Account Name</dt>
                <dd class="mt-1 text-sm text-gray-900">{warehouse.storage_config.account_name}</dd>
              </div>
            {/if}
            {#if warehouse.storage_config.project_id}
              <div>
                <dt class="text-sm font-medium text-gray-700">Project ID</dt>
                <dd class="mt-1 text-sm text-gray-900">{warehouse.storage_config.project_id}</dd>
              </div>
            {/if}
          </dl>
        </div>
      </div>
    </div>

    <!-- Authentication Details -->
    {#if warehouse.use_sts}
      <div class="bg-green-50 border border-green-200 rounded-lg p-6 mb-6">
        <h2 class="text-xl font-semibold mb-4 text-green-900">IAM Role Configuration</h2>
        <dl class="space-y-3">
          {#if warehouse.storage_config.role_arn}
            <div>
              <dt class="text-sm font-medium text-green-700">Role ARN</dt>
              <dd class="mt-1 text-sm text-green-900 font-mono">{warehouse.storage_config.role_arn}</dd>
            </div>
          {/if}
          {#if warehouse.storage_config.external_id}
            <div>
              <dt class="text-sm font-medium text-green-700">External ID</dt>
              <dd class="mt-1 text-sm text-green-900 font-mono">{warehouse.storage_config.external_id}</dd>
            </div>
          {/if}
          {#if warehouse.storage_config.client_id}
            <div>
              <dt class="text-sm font-medium text-green-700">Client ID</dt>
              <dd class="mt-1 text-sm text-green-900 font-mono">{warehouse.storage_config.client_id}</dd>
            </div>
          {/if}
          {#if warehouse.storage_config.tenant_id}
            <div>
              <dt class="text-sm font-medium text-green-700">Tenant ID</dt>
              <dd class="mt-1 text-sm text-green-900 font-mono">{warehouse.storage_config.tenant_id}</dd>
            </div>
          {/if}
        </dl>
        <p class="mt-4 text-sm text-green-700">
          This warehouse uses temporary credentials vended via IAM roles for enhanced security.
        </p>
      </div>
    {:else}
      <div class="bg-yellow-50 border border-yellow-200 rounded-lg p-6 mb-6">
        <h2 class="text-xl font-semibold mb-2 text-yellow-900">Static Credentials</h2>
        <p class="text-sm text-yellow-700">
          This warehouse uses static credentials. Credentials are stored securely and not displayed.
        </p>
      </div>
    {/if}

    <!-- Info Box -->
    <div class="bg-blue-50 border border-blue-200 rounded-lg p-4">
      <p class="text-sm text-blue-800">
        <strong>Note:</strong> This warehouse provides authentication context for catalogs. Individual catalogs can specify their storage location anywhere accessible by these credentials.
      </p>
    </div>
  {/if}
</div>

<!-- Delete Confirmation Dialog -->
<ConfirmDialog
  bind:open={showDeleteDialog}
  title="Delete Warehouse"
  message="Are you sure you want to delete warehouse '{warehouseName}'? This action cannot be undone."
  variant="danger"
  confirmText="Delete"
  loading={deleting}
  on:confirm={handleDelete}
/>
