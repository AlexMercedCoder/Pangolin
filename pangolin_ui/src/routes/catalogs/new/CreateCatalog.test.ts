
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import CreateCatalogPage from './+page.svelte';
import { warehousesApi } from '$lib/api/warehouses';
import { catalogsApi } from '$lib/api/catalogs';

// Mock the API modules
vi.mock('$lib/api/warehouses', () => ({
    warehousesApi: {
        list: vi.fn(),
    }
}));

vi.mock('$lib/api/catalogs', () => ({
    catalogsApi: {
        create: vi.fn(),
    }
}));

describe('CreateCatalogPage', () => {
    beforeEach(() => {
        vi.resetAllMocks();
        // Default mock implementation
        vi.mocked(warehousesApi.list).mockResolvedValue([
            { id: '1', name: 'warehouse-1', use_sts: false, storage_config: { type: 's3' } },
            { id: '2', name: 'warehouse-2', use_sts: false, storage_config: { type: 's3' } }
        ]);
        vi.mocked(catalogsApi.create).mockResolvedValue({ id: 'cat-1', name: 'my-catalog', warehouse_name: '', storage_location: '', properties: {} });
    });

    it('renders the form correctly', async () => {
        render(CreateCatalogPage);
        expect(screen.getByRole('heading', { name: 'Create Catalog' })).toBeInTheDocument();
        // Wait for warehouses to load
        await waitFor(() => expect(warehousesApi.list).toHaveBeenCalled());
    });

    it('shows warning when no warehouse is selected', async () => {
        render(CreateCatalogPage);
        // Initially no warehouse selected
        await waitFor(() => expect(warehousesApi.list).toHaveBeenCalled());

        expect(screen.getByText(/Without a warehouse, you must manually specify storage credentials/i)).toBeInTheDocument();
    });

    it('submit button is enabled when name is provided, even without warehouse', async () => {
        render(CreateCatalogPage);
        await waitFor(() => expect(warehousesApi.list).toHaveBeenCalled());

        const nameInput = screen.getByLabelText(/Name/i);
        const storageInput = screen.getByLabelText(/Storage Location/i);
        const submitButton = screen.getByRole('button', { name: /Create Catalog/i });

        expect(submitButton).toBeDisabled();

        await fireEvent.input(nameInput, { target: { value: 'my-catalog' } });
        // Still disabled because storage location missing
        expect(submitButton).toBeDisabled();

        await fireEvent.input(storageInput, { target: { value: 's3://bucket/path' } });

        // Should be enabled now
        expect(submitButton).not.toBeDisabled();
    });

    it('submits correctly without warehouse', async () => {
        render(CreateCatalogPage);
        await waitFor(() => expect(warehousesApi.list).toHaveBeenCalled());

        const nameInput = screen.getByLabelText(/Name/i);
        const storageInput = screen.getByLabelText(/Storage Location/i);
        const submitButton = screen.getByRole('button', { name: /Create Catalog/i });

        await fireEvent.input(nameInput, { target: { value: 'my-catalog' } });
        await fireEvent.input(storageInput, { target: { value: 's3://bucket/path' } });
        await fireEvent.click(submitButton);

        await waitFor(() => expect(catalogsApi.create).toHaveBeenCalledWith({
            name: 'my-catalog',
            warehouse_name: undefined, 
            storage_location: 's3://bucket/path',
            properties: {}
        }));
    });
});
