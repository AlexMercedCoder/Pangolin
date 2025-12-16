<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { usersApi, type User } from '$lib/api/users';
  import { rolesApi, type Role } from '$lib/api/roles';
  import Button from '$lib/components/ui/Button.svelte';
  import Card from '$lib/components/ui/Card.svelte';
  import { notifications } from '$lib/stores/notifications';
  import RoleAssignment from '$lib/components/rbac/RoleAssignment.svelte';

  let user: User | null = null;
  let allRoles: Role[] = [];
  let assignedRoleIds: string[] = [];
  let loading = true;
  let submitting = false;

  onMount(async () => {
    await loadData();
  });

  async function loadData() {
    loading = true;
    try {
      const userId = $page.params.id;
      if (!userId) {
           goto('/users');
           return;
      }
      // Parallel fetch
      const [userData, rolesData, userRolesData] = await Promise.all([
        usersApi.get(userId),
        rolesApi.list(),
        rolesApi.getUserRoles(userId)
      ]);
      
      user = userData;
      allRoles = rolesData;
      assignedRoleIds = userRolesData.map(r => r.id);

    } catch (error: any) {
      notifications.error(`Failed to load data: ${error.message}`);
      goto('/users');
    }
    loading = false;
  }
  
  async function handleRoleChange(event: CustomEvent) {
      // Optimistic or manual save?
      // Since assign/revoke are atomic operations in backend (usually), 
      // we might want to handle them one by one or batch save.
      // The component gives us the NEW list of IDs.
      // We need to calculate diff.
      const newIds = event.detail.assignedRoleIds as string[];
      
      // Calculate added and removed
      const added = newIds.filter(id => !assignedRoleIds.includes(id));
      const removed = assignedRoleIds.filter(id => !newIds.includes(id));
      
      // Perform updates
      try {
          for (const id of added) {
              await rolesApi.assignToUser(user!.id, id);
          }
          for (const id of removed) {
              await rolesApi.revokeFromUser(user!.id, id);
          }
          assignedRoleIds = newIds;
          notifications.success('Roles updated');
      } catch (e: any) {
          notifications.error('Failed to update roles: ' + e.message);
          // Revert UI? simpler to reload
          await loadData();
      }
  }

</script>

<svelte:head>
  <title>User Details - Pangolin</title>
</svelte:head>

<div class="max-w-6xl mx-auto space-y-6">
  <div class="flex items-center justify-between">
    <h1 class="text-3xl font-bold">User Details</h1>
    {#if user}
        <div>
            <span class="badge variant-filled-primary mr-2">{user.role}</span>
        </div>
    {/if}
  </div>

  {#if loading}
    <div class="flex justify-center p-8">
      <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-500"></div>
    </div>
  {:else if user}
    <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <!-- Sidebar / Info Tests -->
        <div class="lg:col-span-1 space-y-6">
            <Card title="Profile">
                <div class="space-y-4">
                    <div>
                        <label class="label text-sm text-surface-400">Username</label>
                        <div class="font-medium">{user.username}</div>
                    </div>
                    <div>
                        <label class="label text-sm text-surface-400">Email</label>
                        <div class="font-medium">{user.email}</div>
                    </div>
                </div>
            </Card>
        </div>

        <!-- Main Content / Roles -->
        <div class="lg:col-span-2 space-y-6">
            <Card title="Role Assignment">
                <p class="text-sm text-surface-400 mb-4">
                    Assign custom roles to this user. System roles (e.g. TenantAdmin) carry implied permissions.
                </p>
                <RoleAssignment 
                    availableRoles={allRoles} 
                    assignedRoleIds={assignedRoleIds}
                    on:change={handleRoleChange}
                />
            </Card>
        </div>
    </div>
  {:else}
    <div class="alert variant-soft-error">User not found</div>
  {/if}
</div>
