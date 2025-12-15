<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import Card from '$lib/components/ui/Card.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import { tenantsApi, type Tenant, type UpdateTenantRequest } from '$lib/api/tenants';
	import { notifications } from '$lib/stores/notifications';

	let tenant: Tenant | null = null;
	let loading = true;
	let submitting = false;

	// Form data
	let name = '';
	let description = '';
	let errors: Record<string, string> = {};

	$: tenantId = $page.params.id;

	onMount(async () => {
		await loadTenant();
	});

	async function loadTenant() {
		if (!tenantId) return;

		loading = true;
		try {
			tenant = await tenantsApi.get(tenantId);
			// Pre-populate form
			name = tenant.name || '';
			description = tenant.description || '';
		} catch (error: any) {
			notifications.error(`Failed to load tenant: ${error.message}`);
			goto('/tenants');
		}
		loading = false;
	}

	function validateForm(): boolean {
		errors = {};

		if (!name || name.trim().length === 0) {
			errors.name = 'Tenant name is required';
		}

		return Object.keys(errors).length === 0;
	}

	async function handleSubmit() {
		if (!validateForm() || !tenantId) return;

		submitting = true;
		try {
			const updateData: UpdateTenantRequest = {
				name: name.trim(),
			};

			if (description && description.trim()) {
				updateData.description = description.trim();
			}

			await tenantsApi.update(tenantId, updateData);
			notifications.success(`Tenant "${name}" updated successfully`);
			goto(`/tenants/${tenantId}`);
		} catch (error: any) {
			notifications.error(`Failed to update tenant: ${error.message}`);
		}
		submitting = false;
	}
</script>

<svelte:head>
	<title>Edit {tenant?.name || 'Tenant'} - Pangolin</title>
</svelte:head>

<div class="space-y-6">
	<!-- Header -->
	<div>
		<div class="flex items-center gap-3">
			<button
				on:click={() => goto(`/tenants/${tenantId}`)}
				class="text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white"
			>
				← Back
			</button>
			<h1 class="text-3xl font-bold text-gray-900 dark:text-white">
				Edit Tenant: {tenant?.name || 'Loading...'}
			</h1>
		</div>
		<p class="mt-2 text-gray-600 dark:text-gray-400">Update tenant information</p>
	</div>

	{#if loading}
		<Card>
			<div class="flex items-center justify-center py-12">
				<div class="w-8 h-8 border-3 border-primary-600 border-t-transparent rounded-full animate-spin" />
			</div>
		</Card>
	{:else if tenant}
		<Card>
			<form on:submit|preventDefault={handleSubmit} class="space-y-6">
				<div class="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-4">
					<p class="text-sm text-blue-800 dark:text-blue-200">
						<strong>Note:</strong> Tenant ID cannot be changed. The ID is used for API authentication and resource isolation.
					</p>
				</div>

				<Input
					label="Tenant ID"
					value={tenant.id}
					disabled
					helpText="Tenant ID is immutable"
				/>

				<Input
					label="Name"
					bind:value={name}
					error={errors.name}
					required
					placeholder="My Organization"
					helpText="Display name for this tenant"
				/>

				<div class="space-y-2">
					<label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
						Description
					</label>
					<textarea
						bind:value={description}
						rows="3"
						class="block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg shadow-sm focus:outline-none focus:ring-2 focus:ring-primary-500 bg-white dark:bg-gray-800 dark:text-white"
						placeholder="Optional description of this tenant"
					></textarea>
				</div>

				<div class="flex items-center gap-3 pt-4 border-t border-gray-200 dark:border-gray-700">
					<Button
						type="button"
						variant="ghost"
						on:click={() => goto(`/tenants/${tenantId}`)}
						disabled={submitting}
					>
						Cancel
					</Button>
					<Button type="submit" loading={submitting}>
						{submitting ? 'Updating...' : 'Update Tenant'}
					</Button>
				</div>
			</form>
		</Card>
	{/if}
</div>
