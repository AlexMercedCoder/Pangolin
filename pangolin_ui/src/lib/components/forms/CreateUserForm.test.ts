
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import CreateUserForm from './CreateUserForm.svelte';
import { usersApi } from '$lib/api/users';
import { tenantsApi } from '$lib/api/tenants';
import { authStore, isRoot } from '$lib/stores/auth';
import { tenantStore } from '$lib/stores/tenant';
import { readable } from 'svelte/store';

// Mock dependencies
vi.mock('$lib/api/users', () => ({
	usersApi: {
		create: vi.fn()
	}
}));

vi.mock('$lib/api/tenants', () => ({
	tenantsApi: {
		list: vi.fn().mockResolvedValue([])
	}
}));

// Mock stores
vi.mock('$lib/stores/auth', () => ({
	authStore: {
		subscribe: vi.fn()
	},
	isRoot: {
		subscribe: vi.fn()
	}
}));

vi.mock('$lib/stores/tenant', () => ({
	tenantStore: {
		subscribe: vi.fn()
	}
}));

describe('CreateUserForm', () => {
	beforeEach(() => {
		vi.clearAllMocks();
        
        // Default store Mocks
        // Default to NO_AUTH behavior: authEnabled = false
        // This implies NO_AUTH mode where tenant_id should be default
        const mockAuthStore = readable({
            isAuthenticated: true,
            user: { id: 'mock-user', role: 'tenant-admin' },
            authEnabled: false // NO_AUTH mode
        });
        
        (authStore.subscribe as any).mockImplementation(mockAuthStore.subscribe);
        
        const mockIsRoot = readable(false);
        (isRoot.subscribe as any).mockImplementation(mockIsRoot.subscribe);

        const mockTenantStore = readable({
            selectedTenantId: null
        });
        (tenantStore.subscribe as any).mockImplementation(mockTenantStore.subscribe);
	});

	it('should submit correct payload in NO_AUTH mode (auto-set default tenant)', async () => {
		render(CreateUserForm);

		// Fill form
		await fireEvent.input(screen.getByLabelText(/Username/i), { target: { value: 'testuser' } });
		await fireEvent.input(screen.getByLabelText(/Email/i), { target: { value: 'test@example.com' } });
		await fireEvent.input(screen.getByLabelText(/^Password/i), { target: { value: 'Password123!' } });
		await fireEvent.input(screen.getByLabelText(/Confirm Password/i), { target: { value: 'Password123!' } });
        
        // Select Role
        // Note: Select component uses native select, so fireEvent.change should work
        // However, the label text might be tricky if it has *
        const roleSelect = screen.getByLabelText(/Role/i);
        await fireEvent.change(roleSelect, { target: { value: 'TenantUser' } });

		// Submit
		const submitButton = screen.getByRole('button', { name: /Create User/i });
		await fireEvent.click(submitButton);

		// Verify API call
		await waitFor(() => {
			expect(usersApi.create).toHaveBeenCalledTimes(1);
			expect(usersApi.create).toHaveBeenCalledWith({
				username: 'testuser',
				email: 'test@example.com',
				password: 'Password123!',
				role: 'TenantUser',
				tenant_id: '00000000-0000-0000-0000-000000000000' // The critical check
			});
		});
	});
});
