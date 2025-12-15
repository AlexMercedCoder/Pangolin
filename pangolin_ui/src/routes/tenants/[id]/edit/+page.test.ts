import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import TenantEditPage from './+page.svelte';
import { tenantsApi } from '$lib/api/tenants';
import { goto } from '$app/navigation';

vi.mock('$lib/api/tenants');
vi.mock('$app/navigation');
vi.mock('$app/stores', () => ({
	page: {
		subscribe: (fn: any) => {
			fn({ params: { id: 'tenant-123' } });
			return () => {};
		}
	}
}));

describe('Tenant Edit Page', () => {
	const mockTenant = {
		id: 'tenant-123',
		name: 'Test Tenant',
		description: 'Test description',
		created_at: '2024-01-01T00:00:00Z',
		properties: {}
	};

	beforeEach(() => {
		vi.resetAllMocks();
		vi.mocked(tenantsApi.get).mockResolvedValue(mockTenant);
		vi.mocked(tenantsApi.update).mockResolvedValue(mockTenant);
	});

	it('loads and displays tenant data', async () => {
		render(TenantEditPage);

		await waitFor(() => {
			expect(tenantsApi.get).toHaveBeenCalledWith('tenant-123');
		});

		await waitFor(() => {
			expect(screen.getByDisplayValue('Test Tenant')).toBeInTheDocument();
			expect(screen.getByDisplayValue('Test description')).toBeInTheDocument();
		});
	});

	it('validates required name field', async () => {
		render(TenantEditPage);

		await waitFor(() => expect(tenantsApi.get).toHaveBeenCalled());

		const nameInput = screen.getByLabelText(/Name/i);
		await fireEvent.input(nameInput, { target: { value: '' } });

		const submitButton = screen.getByRole('button', { name: /Update Tenant/i });
		await fireEvent.click(submitButton);

		// Should not call update if validation fails
		expect(tenantsApi.update).not.toHaveBeenCalled();
	});

	it('submits update with valid data', async () => {
		render(TenantEditPage);

		await waitFor(() => expect(tenantsApi.get).toHaveBeenCalled());

		const nameInput = screen.getByLabelText(/Name/i);
		await fireEvent.input(nameInput, { target: { value: 'Updated Tenant' } });

		const descInput = screen.getByLabelText(/Description/i);
		await fireEvent.input(descInput, { target: { value: 'Updated description' } });

		const submitButton = screen.getByRole('button', { name: /Update Tenant/i });
		await fireEvent.click(submitButton);

		await waitFor(() => {
			expect(tenantsApi.update).toHaveBeenCalledWith('tenant-123', {
				name: 'Updated Tenant',
				description: 'Updated description'
			});
		});

		expect(goto).toHaveBeenCalledWith('/tenants/tenant-123');
	});

	it('handles empty description', async () => {
		render(TenantEditPage);

		await waitFor(() => expect(tenantsApi.get).toHaveBeenCalled());

		const descInput = screen.getByLabelText(/Description/i);
		await fireEvent.input(descInput, { target: { value: '' } });

		const submitButton = screen.getByRole('button', { name: /Update Tenant/i });
		await fireEvent.click(submitButton);

		await waitFor(() => {
			expect(tenantsApi.update).toHaveBeenCalledWith('tenant-123', {
				name: 'Test Tenant'
				// description should not be included if empty
			});
		});
	});

	it('handles update errors', async () => {
		vi.mocked(tenantsApi.update).mockRejectedValue(new Error('Update failed'));

		render(TenantEditPage);

		await waitFor(() => expect(tenantsApi.get).toHaveBeenCalled());

		const submitButton = screen.getByRole('button', { name: /Update Tenant/i });
		await fireEvent.click(submitButton);

		await waitFor(() => {
			expect(tenantsApi.update).toHaveBeenCalled();
		});

		expect(goto).not.toHaveBeenCalled();
	});

	it('allows canceling edit', async () => {
		render(TenantEditPage);

		await waitFor(() => expect(tenantsApi.get).toHaveBeenCalled());

		const cancelButton = screen.getByRole('button', { name: /Cancel/i });
		await fireEvent.click(cancelButton);

		expect(goto).toHaveBeenCalledWith('/tenants/tenant-123');
		expect(tenantsApi.update).not.toHaveBeenCalled();
	});
});
