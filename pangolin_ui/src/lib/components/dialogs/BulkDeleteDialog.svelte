<script lang="ts">
  import Modal from '$lib/components/ui/Modal.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import { optimizationApi } from '$lib/api/optimization';
  import { createEventDispatcher } from 'svelte';
  import { notifications } from '$lib/stores/notifications';

  export let open = false;
  export let selectedIds: string[] = []; // asset IDs
  export let selectedNames: string[] = []; // friendly names for display

  const dispatch = createEventDispatcher();

  let deleting = false;
  let results: any = null;
  let error: string | null = null;

  async function handleBulkDelete() {
      if (selectedIds.length === 0) return;
      
      deleting = true;
      error = null;
      results = null;

      try {
          const res = await optimizationApi.bulkDeleteAssets(selectedIds);
          results = res;
          
          if (res.failed === 0) {
               notifications.success(`Successfully deleted ${res.succeeded} assets`);
               open = false;
               dispatch('success');
          } else {
               // Stay open to show errors
               notifications.warning(`Deleted ${res.succeeded}, Failed ${res.failed}`);
          }
      } catch (e: any) {
          error = e.message || 'Operation failed';
      } finally {
          deleting = false;
      }
  }

  function handleClose() {
      open = false;
      results = null;
      error = null;
  }
</script>

<Modal bind:open title="Delete Assets" size="lg">
  <div class="space-y-4">
      {#if !results}
          <div class="p-4 bg-red-50 dark:bg-red-900/20 text-red-800 dark:text-red-200 rounded-lg flex items-start gap-3">
              <span class="material-icons text-xl">warning</span>
              <div>
                  <h4 class="font-bold">Warning: Irreversible Action</h4>
                  <p class="text-sm mt-1">
                      You are about to delete <strong>{selectedIds.length}</strong> assets. This will remove all metadata and cannot be undone. 
                      Underlying data files in storage may remain depending on warehouse configuration.
                  </p>
              </div>
          </div>
          
          <div class="max-h-48 overflow-y-auto border border-gray-200 dark:border-gray-700 rounded-md p-2">
              <ul class="space-y-1 text-sm text-gray-700 dark:text-gray-300">
                  {#each selectedNames as name}
                      <li>• {name}</li>
                  {/each}
              </ul>
          </div>
      {:else}
          <!-- Results View -->
          <div class="space-y-4">
              <div class="grid grid-cols-2 gap-4">
                  <div class="p-4 bg-green-50 dark:bg-green-900/20 rounded-lg text-center">
                      <div class="text-2xl font-bold text-green-600 dark:text-green-400">{results.succeeded}</div>
                      <div class="text-sm text-green-800 dark:text-green-200">Succeeded</div>
                  </div>
                   <div class="p-4 bg-red-50 dark:bg-red-900/20 rounded-lg text-center">
                      <div class="text-2xl font-bold text-red-600 dark:text-red-400">{results.failed}</div>
                      <div class="text-sm text-red-800 dark:text-red-200">Failed</div>
                  </div>
              </div>
              
              {#if results.errors && results.errors.length > 0}
                  <div class="p-3 bg-gray-100 dark:bg-gray-800 rounded-lg">
                      <h5 class="text-sm font-semibold mb-2">Errors:</h5>
                      <ul class="text-xs text-red-600 dark:text-red-400 space-y-1">
                           {#each results.errors as err}
                              <li>• {err}</li>
                           {/each}
                      </ul>
                  </div>
              {/if}
          </div>
      {/if}

      {#if error}
          <div class="p-3 bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-300 rounded-lg text-sm">
              {error}
          </div>
      {/if}
  </div>

  <div slot="footer">
      {#if !results}
          <Button variant="secondary" on:click={handleClose} disabled={deleting}>Cancel</Button>
          <Button variant="error" on:click={handleBulkDelete} loading={deleting}>
              Delete {selectedIds.length} Assets
          </Button>
      {:else}
          <Button variant="primary" on:click={() => { dispatch('success'); handleClose(); }}>Close</Button>
      {/if}
  </div>
</Modal>
