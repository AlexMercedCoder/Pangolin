<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import Card from '$lib/components/ui/Card.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
	import EditPermissionsDialog from '$lib/components/dialogs/EditPermissionsDialog.svelte';
	import { rolesApi, type Role } from '$lib/api/roles';
	import { permissionsApi, type Permission, getScopeDisplay } from '$lib/api/permissions';
	import { usersApi } from '$lib/api/users';
	import { notifications } from '$lib/stores/notifications';

	let role: Role | null = null;
	let permissions: Permission[] = [];
	let users: any[] = [];
	let loading = true;
	let showDeleteDialog = false;
	let showPermissionsDialog = false;
	let deleting = false;

	$: roleId = $page.params.id;

	onMount(async () => {
		await loadRole();
		await loadUsers();
	});

	async function loadRole() {
		if (!roleId) return;

		loading = true;
		try {
			role = await rolesApi.get(roleId);
			// Note: Backend doesn't have role-specific permissions endpoint
			// For now, we'll show this is a placeholder for future enhancement
		} catch (error: any) {
			notifications.error(`Failed to load role: ${error.message}`);
			goto('/roles');
		}
		loading = false;
	}

	async function loadUsers() {
		try {
			const allUsers = await usersApi.list();
			// Filter users by role (this would need backend support)
			users = allUsers; // Placeholder
		} catch (error: any) {
			console.error('Failed to load users:', error);
		}
	}

	async function handleDelete() {
		if (!roleId) return;

		deleting = true;
		try {
			await rolesApi.delete(roleId);
			notifications.success(`Role "${role?.name}" deleted successfully`);
			goto('/roles');
		} catch (error: any) {
			notifications.error(`Failed to delete role: ${error.message}`);
		}
		deleting = false;
	}

	function handlePermissionsUpdated() {
		showPermissionsDialog = false;
		notifications.success('Permissions updated successfully');
	}
</script>

<svelte:head>
	<title>{role?.name || 'Role'} - Pangolin</title>
</svelte:head>

<div class="space-y-6">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<div>
			<div class="flex items-center gap-3">
				<button
					on:click={() => goto('/roles')}
					class="text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white"
				>
					← Back
				</button>
				<h1 class="text-3xl font-bold text-gray-900 dark:text-white">
					{role?.name || 'Loading...'}
				</h1>
			</div>
			<p class="mt-2 text-gray-600 dark:text-gray-400">Role details and permissions</p>
		</div>
		<div class="flex items-center gap-3">
			<Button on:click={() => (showPermissionsDialog = true)} disabled={loading}>
				Edit Permissions
			</Button>
			<Button on:click={() => goto(`/roles/${roleId}/edit`)} disabled={loading}>
				Edit Role
			</Button>
			<Button variant="error" on:click={() => (showDeleteDialog = true)} disabled={loading}>
				Delete Role
			</Button>
		</div>
	</div>

	{#if loading}
		<Card>
			<div class="flex items-center justify-center py-12">
				<div class="w-8 h-8 border-3 border-primary-600 border-t-transparent rounded-full animate-spin" />
			</div>
		</Card>
	{:else if role}
		<!-- Details Card -->
		<Card>
			<h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">Details</h3>
			<dl class="grid grid-cols-1 md:grid-cols-2 gap-4">
				<div>
					<dt class="text-sm font-medium text-gray-500 dark:text-gray-400">ID</dt>
					<dd class="mt-1 text-sm text-gray-900 dark:text-white font-mono">{role.id}</dd>
				</div>
				<div>
					<dt class="text-sm font-medium text-gray-500 dark:text-gray-400">Name</dt>
					<dd class="mt-1 text-sm text-gray-900 dark:text-white">{role.name}</dd>
				</div>
				{#if role.description}
					<div class="md:col-span-2">
						<dt class="text-sm font-medium text-gray-500 dark:text-gray-400">Description</dt>
						<dd class="mt-1 text-sm text-gray-900 dark:text-white">{role.description}</dd>
					</div>
				{/if}
				<div>
					<dt class="text-sm font-medium text-gray-500 dark:text-gray-400">Created</dt>
					<dd class="mt-1 text-sm text-gray-900 dark:text-white">
						{new Date(role.created_at).toLocaleString()}
					</dd>
				</div>
			</dl>
		</Card>

		<!-- Permissions Card -->
		<Card>
			<div class="flex items-center justify-between mb-4">
				<h3 class="text-lg font-semibold text-gray-900 dark:text-white">Permissions</h3>
				<Button size="sm" on:click={() => (showPermissionsDialog = true)}>
					Add Permission
				</Button>
			</div>
			<div class="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-4">
				<p class="text-sm text-blue-800 dark:text-blue-200">
					<strong>Note:</strong> Role-based permissions are managed through the permission matrix.
					Click "Edit Permissions" to grant or revoke permissions for this role.
				</p>
			</div>
		</Card>

		<!-- Users with this Role -->
		<Card>
			<h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">
				Users with this Role
			</h3>
			<div class="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-4">
				<p class="text-sm text-yellow-800 dark:text-yellow-200">
					User-role assignment tracking will be available in a future update.
				</p>
			</div>
		</Card>
	{/if}
</div>

<!-- Delete Confirmation Dialog -->
<ConfirmDialog
	bind:open={showDeleteDialog}
	title="Delete Role"
	message="Are you sure you want to delete this role? This will remove all associated permissions and user assignments."
	confirmText={deleting ? 'Deleting...' : 'Delete'}
	variant="danger"
	onConfirm={handleDelete}
	loading={deleting}
/>

<!-- Edit Permissions Dialog -->
{#if role}
	<EditPermissionsDialog
		bind:open={showPermissionsDialog}
		entityType="role"
		entityId={role.id}
		entityName={role.name}
		on:updated={handlePermissionsUpdated}
	/>
{/if}
