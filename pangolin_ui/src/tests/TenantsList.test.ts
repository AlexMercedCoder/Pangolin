
import { render, screen, waitFor } from '@testing-library/svelte';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import TenantsPage from '../routes/tenants/+page.svelte';
import { tenantsApi } from '$lib/api/tenants';

describe('Tenants Page', () => {
    beforeEach(() => {
        vi.mocked(tenantsApi.list).mockResolvedValue([
            { id: 't1', name: 'Tenant One', properties: {} },
            { id: 't2', name: 'Tenant Two', properties: {} }
        ]);
    });

    it('renders tenant list', async () => {
        render(TenantsPage);
        
        // Should show loading state initially or resolve quickly
        await waitFor(() => {
            expect(screen.getByText('Tenant One')).toBeInTheDocument();
            expect(screen.getByText('Tenant Two')).toBeInTheDocument();
        });
    });

    it('shows create tenant button', () => {
        render(TenantsPage);
        expect(screen.getByText('Create Tenant')).toBeInTheDocument();
    });
});
