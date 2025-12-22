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
	import { tenantStore } from '$lib/stores/tenant';

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
		{ value: 'tenant-admin', label: 'Tenant Admin' },
		{ value: 'tenant-user', label: 'Tenant User' }
	];

	// Add Root option only for Root users
	$: allRoleOptions = $isRoot
		? [{ value: 'Root', label: 'Root - Full system access' }, ...roleOptions]
		: roleOptions;

	$: tenantOptions = tenants.map((t) => ({ value: t.id, label: t.name }));

	// Only show tenant select for Root users creating non-Root users
	// In NO_AUTH mode, always use default tenant
	$: showTenantSelect = $isRoot && formData.role && formData.role !== 'Root';

	// Auto-set tenant for non-root users or in NO_AUTH mode
	$: if (!$authStore.authEnabled) {
		// In NO_AUTH mode, use the default tenant
		formData.tenant_id = '00000000-0000-0000-0000-000000000000';
	} else if (!$isRoot) {
        // Fallback to authStore if tenantStore is empty, but protect against undefined
		formData.tenant_id = $tenantStore.selectedTenantId || $authStore.user?.tenant_id;
	}

	async function loadTenants() {
		loadingTenants = true;
		try {
			if ($isRoot) {
				tenants = await tenantsApi.list();
			}
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

		// Tenant validation
		// Root users don't need a tenant
		// TenantAdmins use their current tenant (auto-set)
		// Only validate if we're showing the tenant select
		if (showTenantSelect && !formData.tenant_id) {
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
		tooltip="A unique name for this user to log in with."
		required
		placeholder="john.doe"
	/>

	<Input
		label="Email"
		type="email"
		bind:value={formData.email}
		error={errors.email}
		helpText="User's email address"
		tooltip="Required for password resets and notifications."
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
