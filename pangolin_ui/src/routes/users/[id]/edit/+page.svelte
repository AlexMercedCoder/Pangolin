<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import Card from '$lib/components/ui/Card.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import { usersApi, type User, type UpdateUserRequest } from '$lib/api/users';
	import { notifications } from '$lib/stores/notifications';

	let user: User | null = null;
	let loading = true;
	let submitting = false;

	// Form data
	let email = '';
	let role = '';
	let password = '';
	let confirmPassword = '';
	let errors: Record<string, string> = {};

	$: userId = $page.params.id;

	onMount(async () => {
		await loadUser();
	});

	async function loadUser() {
		if (!userId) return;

		loading = true;
		try {
			user = await usersApi.get(userId);
			// Pre-populate form
			email = user.email || '';
			role = user.role || 'TenantUser';
		} catch (error: any) {
			notifications.error(`Failed to load user: ${error.message}`);
			goto('/users');
		}
		loading = false;
	}

	function validateForm(): boolean {
		errors = {};

		if (!email || !email.includes('@')) {
			errors.email = 'Valid email is required';
		}

		if (password && password !== confirmPassword) {
			errors.confirmPassword = 'Passwords do not match';
		}

		if (password && password.length < 8) {
			errors.password = 'Password must be at least 8 characters';
		}

		return Object.keys(errors).length === 0;
	}

	async function handleSubmit() {
		if (!validateForm() || !userId) return;

		submitting = true;
		try {
			const updateData: UpdateUserRequest = {
				email,
				role,
			};

			// Only include password if it was changed
			if (password) {
				updateData.password = password;
			}

			await usersApi.update(userId, updateData);
			notifications.success(`User "${user?.username}" updated successfully`);
			goto(`/users/${userId}`);
		} catch (error: any) {
			notifications.error(`Failed to update user: ${error.message}`);
		}
		submitting = false;
	}
</script>

<svelte:head>
	<title>Edit {user?.username || 'User'} - Pangolin</title>
</svelte:head>

<div class="space-y-6">
	<!-- Header -->
	<div>
		<div class="flex items-center gap-3">
			<button
				on:click={() => goto(`/users/${userId}`)}
				class="text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white"
			>
				← Back
			</button>
			<h1 class="text-3xl font-bold text-gray-900 dark:text-white">
				Edit User: {user?.username || 'Loading...'}
			</h1>
		</div>
		<p class="mt-2 text-gray-600 dark:text-gray-400">Update user information and permissions</p>
	</div>

	{#if loading}
		<Card>
			<div class="flex items-center justify-center py-12">
				<div class="w-8 h-8 border-3 border-primary-600 border-t-transparent rounded-full animate-spin" />
			</div>
		</Card>
	{:else if user}
		<Card>
			<form on:submit|preventDefault={handleSubmit} class="space-y-6">
				<div class="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-4">
					<p class="text-sm text-blue-800 dark:text-blue-200">
						<strong>Note:</strong> Username cannot be changed. To change a username, create a new user account.
					</p>
				</div>

				<Input
					label="Username"
					value={user.username}
					disabled
					helpText="Username is immutable"
				/>

				<Input
					label="Email"
					type="email"
					bind:value={email}
					error={errors.email}
					required
					placeholder="user@example.com"
				/>

				<div class="space-y-2">
					<label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
						Role
					</label>
					<select
						bind:value={role}
						class="block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg shadow-sm focus:outline-none focus:ring-2 focus:ring-primary-500 bg-white dark:bg-gray-800 dark:text-white"
						disabled={submitting}
					>
						<option value="Root">Root Admin</option>
						<option value="TenantAdmin">Tenant Admin</option>
						<option value="TenantUser">Tenant User</option>
					</select>
					<p class="text-sm text-gray-500 dark:text-gray-400">
						User's permission level within the system
					</p>
				</div>

				<div class="border-t border-gray-200 dark:border-gray-700 pt-6">
					<h3 class="text-lg font-medium text-gray-900 dark:text-white mb-4">
						Change Password (Optional)
					</h3>
					<p class="text-sm text-gray-600 dark:text-gray-400 mb-4">
						Leave blank to keep current password
					</p>

					<div class="space-y-4">
						<Input
							label="New Password"
							type="password"
							bind:value={password}
							error={errors.password}
							placeholder="Leave blank to keep current password"
							helpText="Minimum 8 characters"
						/>

						{#if password}
							<Input
								label="Confirm New Password"
								type="password"
								bind:value={confirmPassword}
								error={errors.confirmPassword}
								placeholder="Re-enter new password"
							/>
						{/if}
					</div>
				</div>

				<div class="flex items-center gap-3 pt-4 border-t border-gray-200 dark:border-gray-700">
					<Button
						type="button"
						variant="ghost"
						on:click={() => goto(`/users/${userId}`)}
						disabled={submitting}
					>
						Cancel
					</Button>
					<Button type="submit" loading={submitting}>
						{submitting ? 'Updating...' : 'Update User'}
					</Button>
				</div>
			</form>
		</Card>
	{/if}
</div>
