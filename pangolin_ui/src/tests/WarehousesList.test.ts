
import { render, screen, waitFor } from '@testing-library/svelte';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import WarehousesPage from '../routes/warehouses/+page.svelte';
import { warehousesApi } from '$lib/api/warehouses';
import { tenantStore } from './mocks';

describe('Warehouses Page', () => {
    beforeEach(() => {
        vi.mocked(warehousesApi.list).mockResolvedValue([
            { 
                id: 'w1', 
                name: 'warehouse-1', 
                tenant_id: 't1',
                use_sts: false,
                storage_config: { type: 's3', bucket: 'test-bucket', region: 'us-east-1' },
                vending_strategy: null
            }
        ]);
        // Set selected tenant to trigger load
        tenantStore.subscribe.mockImplementation((run) => {
            run({ selectedTenantId: 't1', tenants: [], loading: false, error: null });
            return () => {};
        });
    });

    it('renders warehouses list', async () => {
        render(WarehousesPage);
        
        await waitFor(() => {
            expect(screen.getByText('warehouse-1')).toBeInTheDocument();
            expect(screen.getByText('test-bucket')).toBeInTheDocument();
            expect(screen.getByText('S3')).toBeInTheDocument();
        });
    });

    it('shows create warehouse button', () => {
        render(WarehousesPage);
        expect(screen.getByText('Create Warehouse')).toBeInTheDocument();
    });
});
