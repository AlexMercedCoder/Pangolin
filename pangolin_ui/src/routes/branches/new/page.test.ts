import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import CreateBranchPage from './+page.svelte';
import { branchesApi } from '$lib/api/branches';
import { catalogsApi } from '$lib/api/catalogs';
import { goto } from '$app/navigation';

vi.mock('$lib/api/branches');
vi.mock('$lib/api/catalogs');
vi.mock('$app/navigation');

describe('Create Branch Page', () => {
	const mockCatalogs = [
		{ id: 'cat-1', name: 'analytics', warehouse_name: 'wh-1', storage_location: 's3://bucket', properties: {} },
		{ id: 'cat-2', name: 'staging', warehouse_name: 'wh-1', storage_location: 's3://bucket', properties: {} }
	];

	beforeEach(() => {
		vi.resetAllMocks();
		vi.mocked(catalogsApi.list).mockResolvedValue(mockCatalogs);
		vi.mocked(branchesApi.create).mockResolvedValue({
			id: 'br-new',
			name: 'new-branch',
			catalog: 'analytics',
			branch_type: 'experimental',
			assets: [],
			created_at: '2024-01-01T00:00:00Z'
		});
	});

	it('loads and displays catalogs', async () => {
		render(CreateBranchPage);

		await waitFor(() => {
			expect(catalogsApi.list).toHaveBeenCalled();
		});

		await waitFor(() => {
			expect(screen.getByText('analytics')).toBeInTheDocument();
			expect(screen.getByText('staging')).toBeInTheDocument();
		});
	});

	it('validates required branch name', async () => {
		render(CreateBranchPage);

		await waitFor(() => expect(catalogsApi.list).toHaveBeenCalled());

		const submitButton = screen.getByRole('button', { name: /Create Branch/i });
		await fireEvent.click(submitButton);

		// Should not call create if validation fails
		expect(branchesApi.create).not.toHaveBeenCalled();
	});

	it('submits form with valid data', async () => {
		render(CreateBranchPage);

		await waitFor(() => expect(catalogsApi.list).toHaveBeenCalled());

		const nameInput = screen.getByLabelText(/Branch Name/i);
		await fireEvent.input(nameInput, { target: { value: 'feature-branch' } });

		const submitButton = screen.getByRole('button', { name: /Create Branch/i });
		await fireEvent.click(submitButton);

		await waitFor(() => {
			expect(branchesApi.create).toHaveBeenCalledWith(expect.objectContaining({
				name: 'feature-branch',
				catalog: 'analytics',
				branch_type: 'experimental'
			}));
		});

		expect(goto).toHaveBeenCalledWith('/branches');
	});

	it('allows selecting catalog', async () => {
		render(CreateBranchPage);

		await waitFor(() => expect(catalogsApi.list).toHaveBeenCalled());

		const catalogSelect = screen.getByDisplayValue('analytics');
		await fireEvent.change(catalogSelect, { target: { value: 'staging' } });

		const nameInput = screen.getByLabelText(/Branch Name/i);
		await fireEvent.input(nameInput, { target: { value: 'test-branch' } });

		const submitButton = screen.getByRole('button', { name: /Create Branch/i });
		await fireEvent.click(submitButton);

		await waitFor(() => {
			expect(branchesApi.create).toHaveBeenCalledWith(expect.objectContaining({
				catalog: 'staging'
			}));
		});
	});

	it('allows selecting branch type', async () => {
		render(CreateBranchPage);

		await waitFor(() => expect(catalogsApi.list).toHaveBeenCalled());

		const nameInput = screen.getByLabelText(/Branch Name/i);
		await fireEvent.input(nameInput, { target: { value: 'prod-branch' } });

		const typeSelect = screen.getByDisplayValue('Experimental');
		await fireEvent.change(typeSelect, { target: { value: 'production' } });

		const submitButton = screen.getByRole('button', { name: /Create Branch/i });
		await fireEvent.click(submitButton);

		await waitFor(() => {
			expect(branchesApi.create).toHaveBeenCalledWith(expect.objectContaining({
				branch_type: 'production'
			}));
		});
	});

	it('parses comma-separated assets', async () => {
		render(CreateBranchPage);

		await waitFor(() => expect(catalogsApi.list).toHaveBeenCalled());

		const nameInput = screen.getByLabelText(/Branch Name/i);
		await fireEvent.input(nameInput, { target: { value: 'asset-branch' } });

		const assetsInput = screen.getByPlaceholderText(/namespace.table1/i);
		await fireEvent.input(assetsInput, { target: { value: 'ns.table1, ns.table2, ns.table3' } });

		const submitButton = screen.getByRole('button', { name: /Create Branch/i });
		await fireEvent.click(submitButton);

		await waitFor(() => {
			expect(branchesApi.create).toHaveBeenCalledWith(expect.objectContaining({
				assets: ['ns.table1', 'ns.table2', 'ns.table3']
			}));
		});
	});

	it('handles empty assets field', async () => {
		render(CreateBranchPage);

		await waitFor(() => expect(catalogsApi.list).toHaveBeenCalled());

		const nameInput = screen.getByLabelText(/Branch Name/i);
		await fireEvent.input(nameInput, { target: { value: 'no-assets-branch' } });

		const submitButton = screen.getByRole('button', { name: /Create Branch/i });
		await fireEvent.click(submitButton);

		await waitFor(() => {
			const callArgs = vi.mocked(branchesApi.create).mock.calls[0][0];
			expect(callArgs.assets).toBeUndefined();
		});
	});

	it('handles API errors', async () => {
		vi.mocked(branchesApi.create).mockRejectedValue(new Error('Creation failed'));

		render(CreateBranchPage);

		await waitFor(() => expect(catalogsApi.list).toHaveBeenCalled());

		const nameInput = screen.getByLabelText(/Branch Name/i);
		await fireEvent.input(nameInput, { target: { value: 'error-branch' } });

		const submitButton = screen.getByRole('button', { name: /Create Branch/i });
		await fireEvent.click(submitButton);

		await waitFor(() => {
			expect(branchesApi.create).toHaveBeenCalled();
		});

		// Should not navigate on error
		expect(goto).not.toHaveBeenCalled();
	});

	it('allows canceling creation', async () => {
		render(CreateBranchPage);

		await waitFor(() => expect(catalogsApi.list).toHaveBeenCalled());

		const cancelButton = screen.getByRole('button', { name: /Cancel/i });
		await fireEvent.click(cancelButton);

		expect(goto).toHaveBeenCalledWith('/branches');
		expect(branchesApi.create).not.toHaveBeenCalled();
	});
});
