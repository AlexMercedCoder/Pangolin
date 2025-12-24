<script lang="ts">
	import { goto } from '$app/navigation';
	import { tenantsApi, type CreateTenantRequest } from '$lib/api/tenants';
	import { usersApi } from '$lib/api/users';
	import Button from '$lib/components/ui/Button.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import Card from '$lib/components/ui/Card.svelte';
	import { notifications } from '$lib/stores/notifications';
	import Textarea from '$lib/components/ui/Textarea.svelte';

	let loading = false;
	let error = '';

	let name = '';
	let description = '';
	
	// Optional admin user fields
	let createAdmin = false;
	let adminUsername = '';
	let adminPassword = '';
	let adminEmail = '';

	async function handleSubmit() {
		loading = true;
		error = '';

		try {
			const request: CreateTenantRequest = {
				name,
				description,
				properties: {}
			};

			const tenant = await tenantsApi.create(request);
			
			// If admin user fields are provided, create the admin user
			if (createAdmin && adminUsername && adminPassword) {
				try {
					await usersApi.create({
						username: adminUsername,
						password: adminPassword,
						email: adminEmail || undefined,
						role: 'tenant-admin',
						tenant_id: tenant.id
					});
					notifications.success('Tenant and admin user created successfully');
				} catch (adminError: any) {
					notifications.warning(`Tenant created but admin user creation failed: ${adminError.message}`);
				}
			} else {
				notifications.success('Tenant created successfully');
			}
			
			goto('/tenants');
		} catch (e: any) {
			error = e.message || 'Failed to create tenant';
			notifications.error(error);
		} finally {
			loading = false;
		}
	}
</script>

<svelte:head>
	<title>Create Tenant - Pangolin</title>
</svelte:head>

<div class="max-w-2xl mx-auto space-y-6">
	<div>
		<h1 class="text-3xl font-bold text-gray-900 dark:text-white">Create Tenant</h1>
		<p class="mt-2 text-gray-600 dark:text-gray-400">
			Create a new isolated environment for your organization
		</p>
	</div>

	<Card>
		<form on:submit|preventDefault={handleSubmit} class="space-y-6">
			{#if error}
				<div class="p-4 bg-error-50 dark:bg-error-900/20 text-error-700 dark:text-error-200 rounded-lg">
					{error}
				</div>
			{/if}

			<div class="grid gap-6">
				<Input
					label="Name"
					bind:value={name}
					placeholder="my-tenant"
					required
					disabled={loading}
				/>

				<Textarea
					label="Description"
					bind:value={description}
					placeholder="A brief description of this tenant"
					rows={4}
					disabled={loading}
				/>
			</div>

			<!-- Admin User Section -->
			<div class="pt-6 border-t border-gray-200 dark:border-gray-700">
				<label class="flex items-center gap-2 cursor-pointer">
					<input
						type="checkbox"
						bind:checked={createAdmin}
						disabled={loading}
						class="w-4 h-4 text-primary-600 bg-gray-100 border-gray-300 rounded focus:ring-primary-500 dark:focus:ring-primary-600 dark:ring-offset-gray-800 focus:ring-2 dark:bg-gray-700 dark:border-gray-600"
					/>
					<span class="text-sm font-medium text-gray-900 dark:text-white">
						Create initial admin user
					</span>
				</label>
				<p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
					Optionally create a tenant admin user to manage this tenant
				</p>
			</div>

			{#if createAdmin}
				<div class="grid gap-4 pl-6 border-l-2 border-primary-200 dark:border-primary-800">
					<Input
						label="Admin Username"
						bind:value={adminUsername}
						placeholder="admin"
						required={createAdmin}
						disabled={loading}
					/>

					<Input
						label="Admin Password"
						type="password"
						bind:value={adminPassword}
						placeholder="••••••••"
						required={createAdmin}
						disabled={loading}
					/>

					<Input
						label="Admin Email (Optional)"
						type="email"
						bind:value={adminEmail}
						placeholder="admin@example.com"
						disabled={loading}
					/>
				</div>
			{/if}

			<div class="flex justify-end gap-3 pt-6 border-t border-gray-200 dark:border-gray-700">
				<Button
					variant="secondary"
					type="button"
					disabled={loading}
					on:click={() => goto('/tenants')}
				>
					Cancel
				</Button>
				<Button
					variant="primary"
					type="submit"
					{loading}
					disabled={loading || !name || (createAdmin && (!adminUsername || !adminPassword))}
				>
					{loading ? 'Creating...' : 'Create Tenant'}
				</Button>
			</div>
		</form>
	</Card>
</div>
