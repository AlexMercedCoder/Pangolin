import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import CatalogEditPage from './+page.svelte';
import { catalogsApi } from '$lib/api/catalogs';
import { goto } from '$app/navigation';

vi.mock('$lib/api/catalogs');
vi.mock('$app/navigation');
vi.mock('$app/stores', () => ({
	page: {
		subscribe: (fn: any) => {
			fn({ params: { name: 'test-catalog' } });
			return () => {};
		}
	}
}));

describe('Catalog Edit Page', () => {
	const mockCatalog = {
		id: 'cat-1',
		name: 'test-catalog',
		warehouse_name: 'test-warehouse',
		storage_location: 's3://bucket/path',
		properties: {}
	};

	beforeEach(() => {
		vi.resetAllMocks();
		vi.mocked(catalogsApi.get).mockResolvedValue(mockCatalog);
		vi.mocked(catalogsApi.update).mockResolvedValue(mockCatalog);
	});

	it('loads and displays catalog data', async () => {
		render(CatalogEditPage);

		await waitFor(() => {
			expect(catalogsApi.get).toHaveBeenCalledWith('test-catalog');
		});

		await waitFor(() => {
			expect(screen.getByDisplayValue('test-warehouse')).toBeInTheDocument();
			expect(screen.getByDisplayValue('s3://bucket/path')).toBeInTheDocument();
		});
	});

	it('validates required storage location', async () => {
		render(CatalogEditPage);

		await waitFor(() => expect(catalogsApi.get).toHaveBeenCalled());

		const storageInput = screen.getByLabelText(/Storage Location/i);
		await fireEvent.input(storageInput, { target: { value: '' } });

		const submitButton = screen.getByRole('button', { name: /Update Catalog/i });
		await fireEvent.click(submitButton);

		// Should not call update if validation fails
		expect(catalogsApi.update).not.toHaveBeenCalled();
	});

	it('submits update with valid data', async () => {
		render(CatalogEditPage);

		await waitFor(() => expect(catalogsApi.get).toHaveBeenCalled());

		const storageInput = screen.getByLabelText(/Storage Location/i);
		await fireEvent.input(storageInput, { target: { value: 's3://new-bucket/path' } });

		const submitButton = screen.getByRole('button', { name: /Update Catalog/i });
		await fireEvent.click(submitButton);

		await waitFor(() => {
			expect(catalogsApi.update).toHaveBeenCalledWith('test-catalog', {
				storage_location: 's3://new-bucket/path',
				warehouse_name: 'test-warehouse'
			});
		});

		expect(goto).toHaveBeenCalledWith('/catalogs/test-catalog');
	});

	it('handles update errors gracefully', async () => {
		vi.mocked(catalogsApi.update).mockRejectedValue(new Error('Update failed'));

		render(CatalogEditPage);

		await waitFor(() => expect(catalogsApi.get).toHaveBeenCalled());

		const submitButton = screen.getByRole('button', { name: /Update Catalog/i });
		await fireEvent.click(submitButton);

		await waitFor(() => {
			expect(catalogsApi.update).toHaveBeenCalled();
		});

		// Should not navigate on error
		expect(goto).not.toHaveBeenCalled();
	});

	it('allows canceling edit', async () => {
		render(CatalogEditPage);

		await waitFor(() => expect(catalogsApi.get).toHaveBeenCalled());

		const cancelButton = screen.getByRole('button', { name: /Cancel/i });
		await fireEvent.click(cancelButton);

		expect(goto).toHaveBeenCalledWith('/catalogs/test-catalog');
		expect(catalogsApi.update).not.toHaveBeenCalled();
	});
});
