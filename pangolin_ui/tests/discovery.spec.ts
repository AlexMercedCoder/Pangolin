import { test, expect } from '@playwright/test';

test.describe('Discovery (Search) E2E', () => {

    const timestamp = Date.now();
    const warehouseName = `search_wh_${timestamp}`;
    const targetCatalog = `search_cat_${timestamp}`;
    const namespace = `search_ns_${timestamp}`;
    const table = `search_table_${timestamp}`;

    test.beforeEach(async ({ page, request }) => {
        // 1. Create Warehouse via API
        const whRes = await request.post('/api/v1/warehouses', {
            data: {
                name: warehouseName,
                storage_type: 's3',
                config: {
                    bucket: 'warehouse',
                    region: 'us-east-1',
                    access_key: 'minioadmin',
                    secret_key: 'minioadmin',
                    endpoint: 'http://localhost:9000'
                }
            }
        });
        expect(whRes.ok()).toBeTruthy();

        // 2. Create Catalog via API
        const catRes = await request.post('/api/v1/catalogs', {
            data: {
                name: targetCatalog,
                warehouse: warehouseName,
                type: 'pangea'
            }
        });
        expect(catRes.ok()).toBeTruthy();

        // 3. Create Namespace via API (Iceberg REST)
        // Endpoint: /v1/{prefix}/namespaces
        const nsRes = await request.post(`/v1/${targetCatalog}/namespaces`, {
            data: {
                namespace: [namespace]
            }
        });
        expect(nsRes.ok()).toBeTruthy();

        // 4. Create Table via API (Iceberg REST)
        // Endpoint: /v1/{prefix}/namespaces/{namespace}/tables
        const tblRes = await request.post(`/v1/${targetCatalog}/namespaces/${namespace}/tables`, {
            data: {
                name: table,
                schema: {
                    type: "struct",
                    fields: [
                        { id: 1, name: "id", type: "int", required: false },
                        { id: 2, name: "data", type: "string", required: false }
                    ]
                }
            }
        });
        expect(tblRes.ok()).toBeTruthy();

        // 5. Visit Home (to be ready for test)
        await page.goto('/');
        await expect(page).toHaveTitle(/Pangolin/);
        // Wait for potential redirect or load
        await expect(page.getByText('TenantAdmin')).toBeVisible();
    });

    test.fixme('should find created table in global search', async ({ page }) => {
        // 1. Go to Discovery Page (Sidebar link)
        await page.click('a[href="/discovery"]');
        await expect(page).toHaveURL(/\/discovery/);

        // 2. Perform Search
        // Wait for potential eventual consistency or indexing
        await page.waitForTimeout(2000);
        
        await page.getByPlaceholder('Search for datasets, tags, or descriptions...').fill(table);
        await page.click('button:has-text("Search")');

        // 3. Verify Result Card
        // The result is a card with the asset name as an h3 header
        await expect(page.getByRole('heading', { name: table })).toBeVisible();
        await expect(page.getByText('Request Access')).toBeVisible();
        
        // Note: Current impl does not link to details page, so we verify finding it.
    });

    test('should show no results for non-existent asset', async ({ page }) => {
        const query = 'non_existent_asset_12345';
        await page.click('a[href="/discovery"]');
        await page.getByPlaceholder('Search for datasets, tags, or descriptions...').fill(query);
        await page.click('button:has-text("Search")');
        
        await expect(page.getByText(`No assets found matching "${query}"`)).toBeVisible();
    });

});
