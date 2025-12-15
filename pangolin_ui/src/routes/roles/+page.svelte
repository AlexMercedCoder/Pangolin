<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import Card from '$lib/components/ui/Card.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import DataTable from '$lib/components/ui/DataTable.svelte';
	import { rolesApi, type Role } from '$lib/api/roles';
	import { notifications } from '$lib/stores/notifications';

	let roles: Role[] = [];
	let loading = true;

	const columns = [
		{ key: 'name', label: 'Role Name', sortable: true },
		{ key: 'description', label: 'Description', sortable: false },
		{ key: 'created_at', label: 'Created', sortable: true },
	];

	onMount(async () => {
		await loadRoles();
	});

	async function loadRoles() {
		loading = true;
		try {
			roles = await rolesApi.list();
		} catch (error: any) {
			notifications.error(`Failed to load roles: ${error.message}`);
			roles = [];
		}
		loading = false;
	}

	function handleRowClick(event: CustomEvent) {
		const role = event.detail;
		goto(`/roles/${role.id}`);
	}
</script>

<svelte:head>
	<title>Roles - Pangolin</title>
</svelte:head>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-3xl font-bold text-gray-900 dark:text-white">Roles</h1>
			<p class="mt-2 text-gray-600 dark:text-gray-400">
				Manage roles and permissions for your organization
			</p>
		</div>
		<Button on:click={() => goto('/roles/new')}>
			<span class="text-lg mr-2">+</span>
			Create Role
		</Button>
	</div>

	<Card>
		<DataTable
			{columns}
			data={roles}
			{loading}
			emptyMessage="No roles found. Create your first role to manage user permissions."
			searchPlaceholder="Search roles..."
			on:rowClick={handleRowClick}
		>
			<svelte:fragment slot="cell" let:row let:column>
				{#if column.key === 'created_at'}
					<span class="text-sm text-gray-600 dark:text-gray-400">
						{new Date(row.created_at).toLocaleDateString()}
					</span>
				{:else if column.key === 'description'}
					<span class="text-sm text-gray-600 dark:text-gray-400">
						{row.description || '-'}
					</span>
				{:else}
					{row[column.key] ?? '-'}
				{/if}
			</svelte:fragment>
		</DataTable>
	</Card>
</div>
