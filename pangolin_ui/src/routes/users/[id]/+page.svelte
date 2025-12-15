<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { getUser, deleteUser, type User } from '$lib/api/users';
  import Button from '$lib/components/ui/Button.svelte';
  import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
  import { notifications } from '$lib/stores/notifications';

  let user: User | null = null;
  let loading = true;
  let showDeleteDialog = false;
  let deleting = false;

  const userId = $page.params.id;

  async function loadUser() {
    loading = true;
    try {
      user = await getUser(userId);
    } catch (error: any) {
      notifications.error('Failed to load user: ' + error.message);
      goto('/users');
    } finally {
      loading = false;
    }
  }

  function handleEdit() {
    // Navigate to edit page (to be implemented)
    // goto(`/users/${userId}/edit`);
    notifications.info('Edit functionality coming soon');
  }

  async function handleDelete() {
    deleting = true;
    try {
      await deleteUser(userId);
      notifications.success(`User "${user?.username}" deleted successfully`);
      goto('/users');
    } catch (error: any) {
      notifications.error('Failed to delete user: ' + error.message);
      showDeleteDialog = false;
    } finally {
      deleting = false;
    }
  }

  onMount(loadUser);
</script>

<div class="p-6">
  <div class="max-w-4xl mx-auto">
    <!-- Header -->
    <div class="flex items-center justify-between mb-6">
      <div class="flex items-center gap-4">
        <Button variant="ghost" on:click={() => goto('/users')}>
          <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 19l-7-7m0 0l7-7m-7 7h18" />
          </svg>
          Back to Users
        </Button>
        {#if user}
          <h1 class="text-2xl font-bold text-gray-900 dark:text-white">
            {user.username}
          </h1>
        {/if}
      </div>

      {#if user}
        <div class="flex gap-2">
          <Button variant="secondary" on:click={handleEdit}>
            <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
            </svg>
            Edit
          </Button>
          <Button variant="danger" on:click={() => showDeleteDialog = true}>
            <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
            </svg>
            Delete
          </Button>
        </div>
      {/if}
    </div>

    {#if loading}
      <div class="flex justify-center py-12">
        <div class="w-8 h-8 border-4 border-primary-500 border-t-transparent rounded-full animate-spin"></div>
      </div>
    {:else if user}
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 overflow-hidden">
        <div class="p-6 grid grid-cols-1 md:grid-cols-2 gap-8">
          <!-- Account Details -->
          <div>
            <h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">Account Details</h3>
            <dl class="space-y-4">
              <div>
                <dt class="text-sm font-medium text-gray-500 dark:text-gray-400">Username</dt>
                <dd class="mt-1 text-sm text-gray-900 dark:text-white">{user.username}</dd>
              </div>
              <div>
                <dt class="text-sm font-medium text-gray-500 dark:text-gray-400">Email</dt>
                <dd class="mt-1 text-sm text-gray-900 dark:text-white">{user.email}</dd>
              </div>
              <div>
                <dt class="text-sm font-medium text-gray-500 dark:text-gray-400">Role</dt>
                <dd class="mt-1">
                  <span class="inline-flex px-2 py-1 text-xs font-semibold rounded-full 
                    {user.role === 'Root' ? 'bg-purple-100 text-purple-800' : 
                     user.role === 'TenantAdmin' ? 'bg-blue-100 text-blue-800' : 
                     'bg-green-100 text-green-800'}">
                    {user.role}
                  </span>
                </dd>
              </div>
              <div>
                <dt class="text-sm font-medium text-gray-500 dark:text-gray-400">User ID</dt>
                <dd class="mt-1 text-sm font-mono text-gray-900 dark:text-white">{user.id}</dd>
              </div>
            </dl>
          </div>

          <!-- Tenant & System Info -->
          <div>
            <h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">Organization & System</h3>
            <dl class="space-y-4">
              <div>
                <dt class="text-sm font-medium text-gray-500 dark:text-gray-400">Tenant</dt>
                <dd class="mt-1 text-sm text-gray-900 dark:text-white">
                  {#if user.tenant_name}
                    <a href="/tenants/{user.tenant_id}" class="text-primary-600 hover:underline">
                      {user.tenant_name}
                    </a>
                  {:else}
                    <span class="text-gray-400">System / N/A</span>
                  {/if}
                </dd>
              </div>
              <div>
                <dt class="text-sm font-medium text-gray-500 dark:text-gray-400">Created At</dt>
                <dd class="mt-1 text-sm text-gray-900 dark:text-white">
                  {new Date(user.created_at).toLocaleString()}
                </dd>
              </div>
              <div>
                <dt class="text-sm font-medium text-gray-500 dark:text-gray-400">Last Login</dt>
                <dd class="mt-1 text-sm text-gray-900 dark:text-white">
                  {user.last_login ? new Date(user.last_login).toLocaleString() : 'Never'}
                </dd>
              </div>
            </dl>
          </div>
        </div>
      </div>
    {:else}
      <div class="text-center py-12 bg-white dark:bg-gray-800 rounded-lg shadow-sm">
        <p class="text-gray-500 dark:text-gray-400">User not found</p>
      </div>
    {/if}
  </div>
</div>

<ConfirmDialog
  bind:open={showDeleteDialog}
  title="Delete User"
  message="Are you sure you want to delete this user? This action cannot be undone."
  variant="danger"
  confirmText="Delete User"
  loading={deleting}
  on:confirm={handleDelete}
  on:cancel={() => showDeleteDialog = false}
/>
