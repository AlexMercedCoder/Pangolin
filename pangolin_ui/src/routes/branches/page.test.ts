import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import BranchesPage from './+page.svelte';
import { branchesApi } from '$lib/api/branches';
import { catalogsApi } from '$lib/api/catalogs';
import { goto } from '$app/navigation';

vi.mock('$lib/api/branches');
vi.mock('$lib/api/catalogs');
vi.mock('$app/navigation');

describe('Branches List Page', () => {
	const mockCatalogs = [
		{ id: 'cat-1', name: 'analytics', catalog_type: 'Local' as const, warehouse_name: 'wh-1', storage_location: 's3://bucket', properties: {} },
		{ id: 'cat-2', name: 'staging', catalog_type: 'Local' as const, warehouse_name: 'wh-1', storage_location: 's3://bucket', properties: {} }
	];

	const mockBranches = [
		{
			id: 'br-1',
			name: 'main',
			catalog: 'analytics',
			branch_type: 'production' as const,
			assets: ['ns.table1', 'ns.table2'],
			created_at: '2024-01-01T00:00:00Z'
		},
		{
			id: 'br-2',
			name: 'dev',
			catalog: 'analytics',
			from_branch: 'main',
			branch_type: 'experimental' as const,
			assets: ['ns.table1'],
			created_at: '2024-01-02T00:00:00Z'
		},
		{
			id: 'br-3',
			name: 'test',
			catalog: 'staging',
			branch_type: 'experimental' as const,
			assets: [],
			created_at: '2024-01-03T00:00:00Z'
		}
	];

	beforeEach(() => {
		vi.resetAllMocks();
		vi.mocked(catalogsApi.list).mockResolvedValue(mockCatalogs);
		vi.mocked(branchesApi.list).mockResolvedValue(mockBranches);
	});

	it('loads and displays catalogs', async () => {
		render(BranchesPage);

		await waitFor(() => {
			expect(catalogsApi.list).toHaveBeenCalled();
		});

		await waitFor(() => {
			expect(screen.getByDisplayValue('analytics')).toBeInTheDocument();
		});
	});

	it('loads and filters branches by selected catalog', async () => {
		render(BranchesPage);

		await waitFor(() => {
			expect(branchesApi.list).toHaveBeenCalled();
		});

		// Should show branches for analytics catalog
		await waitFor(() => {
			expect(screen.getByText('main')).toBeInTheDocument();
			expect(screen.getByText('dev')).toBeInTheDocument();
		});

		// Should not show staging catalog branches
		expect(screen.queryByText('test')).not.toBeInTheDocument();
	});

	it('filters branches when catalog selection changes', async () => {
		render(BranchesPage);

		await waitFor(() => expect(branchesApi.list).toHaveBeenCalled());

		const catalogSelect = screen.getByDisplayValue('analytics');
		await fireEvent.change(catalogSelect, { target: { value: 'staging' } });

		await waitFor(() => {
			// Should now show staging branch
			expect(screen.getByText('test')).toBeInTheDocument();
		});

		// Should not show analytics branches
		expect(screen.queryByText('main')).not.toBeInTheDocument();
		expect(screen.queryByText('dev')).not.toBeInTheDocument();
	});

	it('displays branch type badges correctly', async () => {
		render(BranchesPage);

		await waitFor(() => expect(branchesApi.list).toHaveBeenCalled());

		await waitFor(() => {
			const productionBadge = screen.getByText('production');
			expect(productionBadge).toBeInTheDocument();
			expect(productionBadge.className).toContain('bg-green');
		});

		const experimentalBadges = screen.getAllByText('experimental');
		expect(experimentalBadges.length).toBeGreaterThan(0);
		expect(experimentalBadges[0].className).toContain('bg-blue');
	});

	it('displays asset counts', async () => {
		render(BranchesPage);

		await waitFor(() => expect(branchesApi.list).toHaveBeenCalled());

		await waitFor(() => {
			expect(screen.getByText('2 assets')).toBeInTheDocument();
			expect(screen.getByText('1 assets')).toBeInTheDocument();
		});
	});

	it('navigates to create page when create button clicked', async () => {
		render(BranchesPage);

		await waitFor(() => expect(catalogsApi.list).toHaveBeenCalled());

		const createButton = screen.getByRole('button', { name: /Create Branch/i });
		await fireEvent.click(createButton);

		expect(goto).toHaveBeenCalledWith('/branches/new');
	});

	it('shows empty state when no branches exist', async () => {
		vi.mocked(branchesApi.list).mockResolvedValue([]);

		render(BranchesPage);

		await waitFor(() => expect(branchesApi.list).toHaveBeenCalled());

		await waitFor(() => {
			expect(screen.getByText(/No branches found/i)).toBeInTheDocument();
		});
	});

	it('handles API errors gracefully', async () => {
		vi.mocked(catalogsApi.list).mockRejectedValue(new Error('API Error'));

		render(BranchesPage);

		await waitFor(() => {
			expect(catalogsApi.list).toHaveBeenCalled();
		});

		// Should not crash, error is handled via notifications
	});
});
