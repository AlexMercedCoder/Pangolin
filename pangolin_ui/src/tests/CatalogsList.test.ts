
import { render, screen, waitFor } from '@testing-library/svelte';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import CatalogsPage from '../routes/catalogs/+page.svelte';
import { catalogsApi } from '$lib/api/catalogs';
import { tenantStore, authStore } from './mocks';

describe('Catalogs Page', () => {
    beforeEach(() => {
        vi.mocked(catalogsApi.list).mockResolvedValue([
            {
                id: 'c1',
                name: 'catalog-1',
                catalog_type: 'Local',
                warehouse_name: 'wh-1',
                storage_location: 's3://test/cat1',
                properties: {}
            }
        ]);

        // Trigger load
         tenantStore.subscribe.mockImplementation((run) => {
            run({ selectedTenantId: 't1', tenants: [], loading: false, error: null });
            return () => {};
        });

        // Set auth role to show create button
        authStore.subscribe.mockImplementation((run) => {
             run({ isAuthenticated: true, user: { role: 'TenantAdmin' }, token: 'token' });
             return () => {};
        });
    });

    it('renders catalogs list', async () => {
        render(CatalogsPage);
        
        await waitFor(() => {
            expect(screen.getByText('catalog-1')).toBeInTheDocument();
            expect(screen.getByText('Local')).toBeInTheDocument();
            expect(screen.getByText('s3://test/cat1')).toBeInTheDocument();
        });
    });

    it('shows create catalog button for Admin', () => {
        render(CatalogsPage);
        expect(screen.getByText('Create Catalog')).toBeInTheDocument();
    });
});
