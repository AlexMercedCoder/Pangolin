<script lang="ts">
	import { createEventDispatcher, onMount } from 'svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import { rolesApi, type Role } from '$lib/api/roles';
	import { notifications } from '$lib/stores/notifications';

	export let open = false;
	export let userId: string;
	export let userName: string;
	export let currentRoles: string[] = []; // Array of role IDs

	const dispatch = createEventDispatcher();

	let roles: Role[] = [];
	let selectedRoles: Set<string> = new Set(currentRoles);
	let loading = true;
	let submitting = false;

	onMount(async () => {
		if (open) {
			await loadRoles();
		}
	});

	$: if (open) {
		loadRoles();
		selectedRoles = new Set(currentRoles);
	}

	async function loadRoles() {
		loading = true;
		try {
			roles = await rolesApi.list();
		} catch (error: any) {
			notifications.error(`Failed to load roles: ${error.message}`);
		}
		loading = false;
	}

	function toggleRole(roleId: string) {
		if (selectedRoles.has(roleId)) {
			selectedRoles.delete(roleId);
		} else {
			selectedRoles.add(roleId);
		}
		selectedRoles = selectedRoles; // Trigger reactivity
	}

	async function handleSave() {
		submitting = true;
		try {
			// Determine which roles to assign and revoke
			const currentSet = new Set(currentRoles);
			const toAssign = Array.from(selectedRoles).filter(id => !currentSet.has(id));
			const toRevoke = currentRoles.filter(id => !selectedRoles.has(id));

			// Assign new roles
			for (const roleId of toAssign) {
				await rolesApi.assignToUser(userId, roleId);
			}

			// Revoke removed roles
			for (const roleId of toRevoke) {
				await rolesApi.revokeFromUser(userId, roleId);
			}

			notifications.success('User roles updated successfully');
			dispatch('updated', { roles: Array.from(selectedRoles) });
			open = false;
		} catch (error: any) {
			notifications.error(`Failed to update roles: ${error.message}`);
		}
		submitting = false;
	}

	function handleClose() {
		open = false;
	}
</script>

{#if open}
	<div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
		<div class="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-2xl w-full max-h-[80vh] overflow-hidden flex flex-col">
			<!-- Header -->
			<div class="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
				<h2 class="text-xl font-semibold text-gray-900 dark:text-white">
					Edit Roles: {userName}
				</h2>
			</div>

			<!-- Content -->
			<div class="flex-1 overflow-y-auto p-6">
				{#if loading}
					<div class="flex items-center justify-center py-8">
						<div class="w-6 h-6 border-2 border-primary-600 border-t-transparent rounded-full animate-spin" />
					</div>
				{:else if roles.length === 0}
					<p class="text-gray-600 dark:text-gray-400 text-center py-8">
						No roles available
					</p>
				{:else}
					<div class="space-y-3">
						<p class="text-sm text-gray-600 dark:text-gray-400 mb-4">
							Select the roles to assign to this user
						</p>
						{#each roles as role}
							<label class="flex items-start gap-3 p-3 bg-gray-50 dark:bg-gray-900 rounded-lg cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors">
								<input
									type="checkbox"
									checked={selectedRoles.has(role.id)}
									on:change={() => toggleRole(role.id)}
									class="mt-1 rounded border-gray-300 text-primary-600 focus:ring-primary-500"
								/>
								<div class="flex-1">
									<div class="font-medium text-gray-900 dark:text-white">
										{role.name}
									</div>
									{#if role.description}
										<div class="text-sm text-gray-600 dark:text-gray-400 mt-1">
											{role.description}
										</div>
									{/if}
								</div>
							</label>
						{/each}
					</div>
				{/if}
			</div>

			<!-- Footer -->
			<div class="px-6 py-4 border-t border-gray-200 dark:border-gray-700 flex justify-end gap-3">
				<Button variant="ghost" on:click={handleClose} disabled={submitting}>
					Cancel
				</Button>
				<Button on:click={handleSave} loading={submitting} disabled={loading}>
					{submitting ? 'Saving...' : 'Save Changes'}
				</Button>
			</div>
		</div>
	</div>
{/if}
