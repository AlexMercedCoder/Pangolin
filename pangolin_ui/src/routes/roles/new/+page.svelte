<script lang="ts">
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import Card from '$lib/components/ui/Card.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import { rolesApi, type CreateRoleRequest } from '$lib/api/roles';
	import { tenantsApi } from '$lib/api/tenants';
	import { authStore } from '$lib/stores/auth';
	import { notifications } from '$lib/stores/notifications';
	import { get } from 'svelte/store';

	let loading = false;
	let tenants: any[] = [];
	let tenantsLoading = true;

	// Form data
	let name = '';
	let description = '';
	let tenantId = '';
	let errors: Record<string, string> = {};

	const auth = get(authStore);
	const isRoot = auth.user?.role === 'Root';

	onMount(async () => {
		if (isRoot) {
			await loadTenants();
		} else {
			// Use current user's tenant
			tenantId = auth.user?.tenant_id || '';
			tenantsLoading = false;
		}
	});

	async function loadTenants() {
		tenantsLoading = true;
		try {
			tenants = await tenantsApi.list();
			if (tenants.length > 0) {
				tenantId = tenants[0].id;
			}
		} catch (error: any) {
			notifications.error(`Failed to load tenants: ${error.message}`);
		}
		tenantsLoading = false;
	}

	function validateForm(): boolean {
		errors = {};

		if (!name || name.trim().length === 0) {
			errors.name = 'Role name is required';
		}

		if (!tenantId) {
			errors.tenantId = 'Tenant is required';
		}

		return Object.keys(errors).length === 0;
	}

	async function handleSubmit() {
		if (!validateForm()) return;

		loading = true;
		try {
			const data: CreateRoleRequest = {
				name: name.trim(),
				description: description.trim() || undefined,
				'tenant-id': tenantId,
			};

			await rolesApi.create(data);
			notifications.success(`Role "${name}" created successfully`);
			goto('/roles');
		} catch (error: any) {
			notifications.error(`Failed to create role: ${error.message}`);
		}
		loading = false;
	}
</script>

<svelte:head>
	<title>Create Role - Pangolin</title>
</svelte:head>

<div class="max-w-3xl mx-auto space-y-6">
	<div>
		<div class="flex items-center gap-3">
			<button
				on:click={() => goto('/roles')}
				class="text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white"
			>
				← Back
			</button>
			<h1 class="text-3xl font-bold text-gray-900 dark:text-white">Create Role</h1>
		</div>
		<p class="mt-2 text-gray-600 dark:text-gray-400">
			Create a new role for managing user permissions
		</p>
	</div>

	<Card>
		<form on:submit|preventDefault={handleSubmit} class="space-y-6">
			<div class="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-4">
				<h3 class="text-sm font-medium text-blue-800 dark:text-blue-200 mb-2">
					About Roles
				</h3>
				<p class="text-sm text-blue-700 dark:text-blue-300">
					Roles are collections of permissions that can be assigned to users. After creating a role,
					you can assign specific permissions to control access to catalogs, namespaces, and tables.
				</p>
			</div>

			<Input
				label="Role Name"
				bind:value={name}
				error={errors.name}
				required
				placeholder="Data Analyst"
				helpText="A descriptive name for this role"
			/>

			<div class="space-y-2">
				<label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
					Description
				</label>
				<textarea
					bind:value={description}
					rows="3"
					placeholder="Optional description of this role's purpose"
					class="block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg shadow-sm focus:outline-none focus:ring-2 focus:ring-primary-500 bg-white dark:bg-gray-800 dark:text-white"
					disabled={loading}
				></textarea>
			</div>

			{#if isRoot}
				<div class="space-y-2">
					<label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
						Tenant
						<span class="text-error-500">*</span>
					</label>
					<select
						bind:value={tenantId}
						disabled={tenantsLoading || loading}
						class="block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg shadow-sm focus:outline-none focus:ring-2 focus:ring-primary-500 bg-white dark:bg-gray-800 dark:text-white {errors.tenantId ? 'border-error-500' : ''}"
					>
						{#if tenantsLoading}
							<option>Loading tenants...</option>
						{:else if tenants.length === 0}
							<option>No tenants available</option>
						{:else}
							{#each tenants as tenant}
								<option value={tenant.id}>{tenant.name}</option>
							{/each}
						{/if}
					</select>
					{#if errors.tenantId}
						<p class="text-sm text-error-600 dark:text-error-400">{errors.tenantId}</p>
					{/if}
				</div>
			{/if}

			<div class="flex items-center gap-3 pt-4 border-t border-gray-200 dark:border-gray-700">
				<Button
					type="button"
					variant="ghost"
					on:click={() => goto('/roles')}
					disabled={loading}
				>
					Cancel
				</Button>
				<Button type="submit" loading={loading}>
					{loading ? 'Creating...' : 'Create Role'}
				</Button>
			</div>
		</form>
	</Card>
</div>
