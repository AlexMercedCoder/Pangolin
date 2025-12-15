<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import DataTable from '$lib/components/ui/DataTable.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
	import CreateUserForm from '$lib/components/forms/CreateUserForm.svelte';
	import { usersApi, type User } from '$lib/api/users';
	import { authStore } from '$lib/stores/auth';
	import { tenantStore } from '$lib/stores/tenant';
	import { notifications } from '$lib/stores/notifications';

	let users: User[] = [];
	let loading = true;
	let showCreateModal = false;
	let showDeleteDialog = false;
	let selectedUser: User | null = null;
	let searchQuery = '';
	let roleFilter = 'all';

	// Reload when tenant changes
	$: if ($tenantStore.selectedTenantId || $tenantStore.selectedTenantId === null) {
		loadUsers();
	}

	const roleColors: Record<string, string> = {
		root: 'bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200',
		'tenant-admin': 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200',
		'tenant-user': 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200'
	};

	const columns = [
		{ key: 'username', label: 'Username', sortable: true },
		{ key: 'email', label: 'Email', sortable: true },
		{ key: 'role', label: 'Role', sortable: true },
		{ key: 'tenant_name', label: 'Tenant', sortable: true },
		{ key: 'created_at', label: 'Created', sortable: true },
		{ key: 'actions', label: 'Actions', sortable: false }
	];

	$: filteredUsers = users.filter((user) => {
		const matchesSearch =
			searchQuery === '' ||
			user.username.toLowerCase().includes(searchQuery.toLowerCase()) ||
			user.email.toLowerCase().includes(searchQuery.toLowerCase());

		const matchesRole = roleFilter === 'all' || user.role === roleFilter;

		return matchesSearch && matchesRole;
	});

	$: tableData = filteredUsers.map((user) => ({
		...user,
		tenant_name: user.tenant_name || 'N/A',
		created_at: new Date(user.created_at).toLocaleDateString(),
		role_badge: user.role,
		actions_data: user
	}));

	async function loadUsers() {
		loading = true;
		try {
			users = await usersApi.list();
		} catch (error: any) {
			notifications.error('Failed to load users: ' + error.message);
		} finally {
			loading = false;
		}
	}

	function handleView(user: User) {
		goto(`/users/${user.id}`);
	}

	function handleDelete(user: User) {
		selectedUser = user;
		showDeleteDialog = true;
	}

	async function confirmDelete() {
		if (!selectedUser) return;

		try {
			await usersApi.delete(selectedUser.id);
			notifications.success(`User "${selectedUser.username}" deleted successfully`);
			showDeleteDialog = false;
			selectedUser = null;
			await loadUsers();
		} catch (error: any) {
			notifications.error('Failed to delete user: ' + error.message);
		}
	}

	function handleCreateSuccess() {
		showCreateModal = false;
		loadUsers();
		notifications.success('User created successfully');
	}

	onMount(loadUsers);
</script>

<svelte:head>
	<title>Users - Pangolin</title>
</svelte:head>

<div class="p-6">
	<div class="flex justify-between items-center mb-6">
		<div>
			<h1 class="text-3xl font-bold text-gray-900 dark:text-white">Users</h1>
			<p class="text-gray-600 dark:text-gray-400 mt-1">Manage user accounts and permissions</p>
		</div>

		{#if $authStore.user?.role === 'Root' || $authStore.user?.role === 'TenantAdmin'}
			<Button on:click={() => (showCreateModal = true)}>
				<svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M12 4v16m8-8H4"
					/>
				</svg>
				Create User
			</Button>
		{/if}
	</div>

	<!-- Filters -->
	<div class="bg-white dark:bg-gray-800 rounded-lg shadow-sm p-4 mb-4">
		<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
			<div>
				<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
					Search
				</label>
				<input
					type="text"
					bind:value={searchQuery}
					placeholder="Search by username or email..."
					class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg
						bg-white dark:bg-gray-800 text-gray-900 dark:text-white
						focus:ring-2 focus:ring-primary-500 focus:border-transparent"
				/>
			</div>

			<div>
				<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
					Filter by Role
				</label>
				<select
					bind:value={roleFilter}
					class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg
						bg-white dark:bg-gray-800 text-gray-900 dark:text-white
						focus:ring-2 focus:ring-primary-500 focus:border-transparent"
				>
					<option value="all">All Roles</option>
					<option value="root">Root</option>
					<option value="tenant-admin">Tenant Admin</option>
					<option value="tenant-user">Tenant User</option>
				</select>
			</div>
		</div>
	</div>

	<!-- Users Table -->
	<DataTable {columns} data={tableData} {loading}>
		<svelte:fragment slot="cell" let:row let:column>
			{#if column.key === 'role'}
				<span class="inline-flex px-2 py-1 text-xs font-semibold rounded-full {roleColors[row.role_badge]}">
					{row.role_badge}
				</span>
			{:else if column.key === 'actions'}
				<div class="flex gap-2">
					<button
						on:click={() => handleView(row.actions_data)}
						class="text-primary-600 hover:text-primary-800 dark:text-primary-400 dark:hover:text-primary-300"
						title="View Details"
					>
						<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
							/>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"
							/>
						</svg>
					</button>

					{#if $authStore.user?.role === 'Root' || ($authStore.user?.role === 'TenantAdmin' && row.role !== 'Root')}
						<button
							on:click={() => handleDelete(row.actions_data)}
							class="text-red-600 hover:text-red-800 dark:text-red-400 dark:hover:text-red-300"
							title="Delete User"
						>
							<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									stroke-width="2"
									d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
								/>
							</svg>
						</button>
					{/if}
				</div>
			{:else}
				{row[column.key]}
			{/if}
		</svelte:fragment>
	</DataTable>
</div>

<!-- Create User Modal -->
<Modal bind:open={showCreateModal} title="Create User">
	<CreateUserForm on:success={handleCreateSuccess} on:cancel={() => (showCreateModal = false)} />
</Modal>

<!-- Delete Confirmation Dialog -->
<ConfirmDialog
	bind:open={showDeleteDialog}
	title="Delete User"
	message="Are you sure you want to delete user '{selectedUser?.username}'? This action cannot be undone."
	confirmText="Delete"
	variant="danger"
	on:confirm={confirmDelete}
	on:cancel={() => {
		showDeleteDialog = false;
		selectedUser = null;
	}}
/>
