<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { tenantsApi, type Tenant } from '$lib/api/tenants';
	import { notifications } from '$lib/stores/notifications';
	import DataTable from '$lib/components/ui/DataTable.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Card from '$lib/components/ui/Card.svelte';

	let tenants: Tenant[] = [];
	let loading = true;

	const columns = [
		{ key: 'name', label: 'Name', sortable: true },
		{ key: 'id', label: 'ID', sortable: true, width: '300px' },
	];

	onMount(async () => {
		await loadTenants();
	});

	async function loadTenants() {
		loading = true;
		try {
			tenants = await tenantsApi.list();
		} catch (error: any) {
			notifications.error(`Failed to load tenants: ${error.message}`);
			tenants = [];
		} finally {
			loading = false;
		}
	}

	function handleRowClick(event: CustomEvent) {
		// Future: Navigate to tenant details
		// const tenant = event.detail;
		// goto(`/tenants/${tenant.id}`);
	}
</script>

<svelte:head>
	<title>Tenants - Pangolin</title>
</svelte:head>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-3xl font-bold text-gray-900 dark:text-white">Tenants</h1>
			<p class="mt-2 text-gray-600 dark:text-gray-400">
				Manage multi-tenant isolation and organization
			</p>
		</div>
		<Button on:click={() => goto('/tenants/new')}>
			<span class="text-lg mr-2">+</span>
			Create Tenant
		</Button>
	</div>

	<Card>
		<DataTable
			{columns}
			data={tenants}
			{loading}
			emptyMessage="No tenants found. Create your first tenant to get started."
			searchPlaceholder="Search tenants..."
			on:rowClick={handleRowClick}
		>
			<svelte:fragment slot="cell" let:row let:column>
				{#if column.key === 'id'}
					<span class="font-mono text-sm">{row.id}</span>
				{:else}
					{row[column.key] ?? '-'}
				{/if}
			</svelte:fragment>
		</DataTable>
	</Card>
</div>
