<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { onMount } from 'svelte';
	import Card from '$lib/components/ui/Card.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import { rolesApi, type Role, type UpdateRoleRequest } from '$lib/api/roles';
	import { notifications } from '$lib/stores/notifications';

	let role: Role | null = null;
	let loading = true;
	let submitting = false;

	// Form data
	let name = '';
	let description = '';
	let errors: Record<string, string> = {};

	let roleId: string; // Changed to be assigned in loadData

	onMount(async () => {
		await loadData();
	});

	async function loadData() {
        roleId = $page.params.id || '';
        if (!roleId) {
            notifications.error("Role ID missing");
            goto('/roles');
            return;
        }

		loading = true; // Keep loading state management
        try {
            const fetchedRole = await rolesApi.get(roleId); // Use a temporary variable for the fetched role
			role = fetchedRole; // Assign to the global 'role' variable
			name = role.name;
			description = role.description || '';
		} catch (error: any) {
			notifications.error(`Failed to load role: ${error.message}`);
			goto('/roles');
		}
		loading = false; // Keep loading state management
	}

	function validateForm(): boolean {
		errors = {};

		if (!name || name.trim().length === 0) {
			errors.name = 'Role name is required';
		}

		return Object.keys(errors).length === 0;
	}

	async function handleSubmit() {
		if (!validateForm()) return;

		submitting = true;
		try {
			const request: UpdateRoleRequest = {
				name: name.trim(),
			};

			if (description && description.trim()) {
				request.description = description.trim();
			}

			await rolesApi.update(roleId, request);
			notifications.success(`Role "${name}" updated successfully`);
			goto(`/roles/${roleId}`);
		} catch (error: any) {
			notifications.error(`Failed to update role: ${error.message}`);
		}
		submitting = false;
	}


</script>

<svelte:head>
	<title>Edit Role - Pangolin</title>
</svelte:head>

<div class="max-w-3xl mx-auto space-y-6">
	<div>
		<div class="flex items-center gap-3">
			<button
				on:click={() => goto(`/roles/${roleId}`)}
				class="text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white"
			>
				← Back
			</button>
			<h1 class="text-3xl font-bold text-gray-900 dark:text-white">Edit Role</h1>
		</div>
		<p class="mt-2 text-gray-600 dark:text-gray-400">
			Update role information
		</p>
	</div>

	{#if loading}
		<Card>
			<div class="flex items-center justify-center py-12">
				<div class="w-8 h-8 border-3 border-primary-600 border-t-transparent rounded-full animate-spin" />
			</div>
		</Card>
	{:else if role}
		<Card>
			<form on:submit|preventDefault={handleSubmit} class="space-y-6">
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
						disabled={submitting}
					></textarea>
				</div>

				<div class="flex items-center gap-3 pt-4 border-t border-gray-200 dark:border-gray-700">
					<Button
						type="button"
						variant="ghost"
						on:click={() => goto(`/roles/${roleId}`)}
						disabled={submitting}
					>
						Cancel
					</Button>
					<Button type="submit" loading={submitting}>
						{submitting ? 'Updating...' : 'Update Role'}
					</Button>
				</div>
			</form>
		</Card>
	{/if}
</div>
