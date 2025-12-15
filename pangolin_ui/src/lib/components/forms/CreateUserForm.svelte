```
<script lang="ts">
	import { createEventDispatcher, onMount } from 'svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import Select from '$lib/components/ui/Select.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import PasswordStrength from '$lib/components/ui/PasswordStrength.svelte';
	import { usersApi, type CreateUserRequest } from '$lib/api/users';
	import { tenantsApi, type Tenant } from '$lib/api/tenants';
	import { authStore, isRoot } from '$lib/stores/auth';

	const dispatch = createEventDispatcher();

	let formData: CreateUserRequest = {
		username: '',
		email: '',
		password: '',
		role: '',
		tenant_id: undefined
	};

	let confirmPassword = '';
	let errors: Record<string, string> = {};
	let submitting = false;
	let tenants: Tenant[] = [];
	let loadingTenants = true;

	const roleOptions = [
		{ value: 'TenantUser', label: 'Tenant User - Basic access to assigned resources' },
		{ value: 'TenantAdmin', label: 'Tenant Admin - Manage tenant resources and users' }
	];

	// Add Root option only for Root users
	$: allRoleOptions = $isRoot
		? [{ value: 'Root', label: 'Root - Full system access' }, ...roleOptions]
		: roleOptions;

	$: tenantOptions = tenants.map((t) => ({ value: t.id, label: t.name }));

	$: showTenantSelect = formData.role && formData.role !== 'Root';

	async function loadTenants() {
		loadingTenants = true;
		try {
			tenants = await tenantsApi.list();
		} catch (error) {
			console.error('Failed to load tenants:', error);
		} finally {
			loadingTenants = false;
		}
	}

	function validateForm(): boolean {
		errors = {};

		// Username validation
		if (!formData.username || formData.username.length < 3) {
			errors.username = 'Username must be at least 3 characters';
		} else if (!/^[a-zA-Z0-9_-]+$/.test(formData.username)) {
			errors.username = 'Username can only contain letters, numbers, dashes, and underscores';
		}

		// Email validation
		if (!formData.email) {
			errors.email = 'Email is required';
		} else if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(formData.email)) {
			errors.email = 'Please enter a valid email address';
		}

		// Password validation
		if (!formData.password) {
			errors.password = 'Password is required';
		} else if (formData.password.length < 8) {
			errors.password = 'Password must be at least 8 characters';
		} else if (!/[A-Z]/.test(formData.password)) {
			errors.password = 'Password must contain at least one uppercase letter';
		} else if (!/[a-z]/.test(formData.password)) {
			errors.password = 'Password must contain at least one lowercase letter';
		} else if (!/[0-9]/.test(formData.password)) {
			errors.password = 'Password must contain at least one number';
		}

		// Confirm password validation
		if (formData.password !== confirmPassword) {
			errors.confirmPassword = 'Passwords do not match';
		}

		// Role validation
		if (!formData.role) {
			errors.role = 'Please select a role';
		}

		// Tenant validation (for non-Root users)
		if (formData.role !== 'Root' && !formData.tenant_id) {
			errors.tenant_id = 'Please select a tenant';
		}

		return Object.keys(errors).length === 0;
	}

	async function handleSubmit() {
		if (!validateForm()) {
			return;
		}

		submitting = true;
		try {
			await usersApi.create(formData);
			dispatch('success');
		} catch (error: any) {
			errors.submit = error.message || 'Failed to create user';
		} finally {
			submitting = false;
		}
	}

	function handleCancel() {
		dispatch('cancel');
	}

	onMount(loadTenants);
</script>

<form on:submit|preventDefault={handleSubmit} class="space-y-4">
	<Input
		label="Username"
		bind:value={formData.username}
		error={errors.username}
		helpText="Used for login (alphanumeric, dashes, underscores only)"
		required
		placeholder="john.doe"
	/>

	<Input
		label="Email"
		type="email"
		bind:value={formData.email}
		error={errors.email}
		helpText="User's email address"
		required
		placeholder="john@example.com"
	/>

	<Input
		label="Password"
		type="password"
		bind:value={formData.password}
		error={errors.password}
		helpText="Minimum 8 characters with uppercase, lowercase, and number"
		required
	/>

	<PasswordStrength password={formData.password} />

	<Input
		label="Confirm Password"
		type="password"
		bind:value={confirmPassword}
		error={errors.confirmPassword}
		required
	/>

	<Select
		label="Role"
		bind:value={formData.role}
		options={allRoleOptions}
		error={errors.role}
		helpText="User's access level"
		required
		placeholder="Select a role..."
	/>

	{#if showTenantSelect}
		<Select
			label="Tenant"
			bind:value={formData.tenant_id}
			options={tenantOptions}
			error={errors.tenant_id}
			helpText="Tenant this user belongs to"
			required
			disabled={loadingTenants}
			placeholder={loadingTenants ? 'Loading tenants...' : 'Select a tenant...'}
		/>
	{/if}

	{#if errors.submit}
		<div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-3">
			<p class="text-sm text-red-600 dark:text-red-400">{errors.submit}</p>
		</div>
	{/if}

	<div class="flex gap-2 justify-end pt-4">
		<Button variant="ghost" on:click={handleCancel} disabled={submitting}>Cancel</Button>
		<Button type="submit" loading={submitting}>Create User</Button>
	</div>
</form>
