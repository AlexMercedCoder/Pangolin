<script lang="ts">
	import { goto } from '$app/navigation';
	import { tenantsApi, type CreateTenantRequest } from '$lib/api/tenants';
	import Button from '$lib/components/ui/Button.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import Card from '$lib/components/ui/Card.svelte';
	import { notifications } from '$lib/stores/notifications';
	import Textarea from '$lib/components/ui/Textarea.svelte';

	let loading = false;
	let error = '';

	let name = '';
	let description = '';

	async function handleSubmit() {
		loading = true;
		error = '';

		try {
			const request: CreateTenantRequest = {
				name,
				description,
				properties: {}
			};

			await tenantsApi.create(request);
			notifications.success('Tenant created successfully');
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
					disabled={loading || !name}
				>
					{loading ? 'Creating...' : 'Create Tenant'}
				</Button>
			</div>
		</form>
	</Card>
</div>
