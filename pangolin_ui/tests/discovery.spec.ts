import { test, expect } from '@playwright/test';

test.describe('Discovery (Search) E2E', () => {

    const timestamp = Date.now();
    const warehouseName = `search_wh_${timestamp}`;
    const targetCatalog = `search_cat_${timestamp}`;
    const namespace = `search_ns_${timestamp}`;
    const table = `search_table_${timestamp}`;

    test.beforeEach(async ({ page, request }) => {
        // 1. Setup Data via API (Directly to Backend to avoid UI flakiness/proxy issues)
        // Using the variables defined at the describe block level
        
        // Create Warehouse
        await request.post('http://localhost:8080/api/v1/warehouses', {
            data: {
                name: warehouseName,
                storage_config: { "type": "s3", "location": "s3://bucket" },
                storage_type: "s3"
            }
        });

        // Create Catalog
        await request.post('http://localhost:8080/api/v1/catalogs', {
            data: {
                name: targetCatalog,
                warehouse_name: warehouseName,
                catalog_type: "iceberg",
                storage_location: `s3://${warehouseName}/${targetCatalog}`
            }
        });

        // Create Namespace (Iceberg REST)
        // Note: Iceberg REST uses /v1/{catalog}/namespaces
        await request.post(`http://localhost:8080/v1/${targetCatalog}/namespaces`, {
            data: {
                namespace: [namespace]
            }
        });

        // Create Table (Iceberg REST)
        await request.post(`http://localhost:8080/v1/${targetCatalog}/namespaces/${namespace}/tables`, {
            data: {
                name: table,
                schema: {
                    type: "struct",
                    fields: [
                        { id: 1, name: "id", type: "int", required: true },
                        { id: 2, name: "data", type: "string", required: false }
                    ]
                },
                location: `s3://${warehouseName}/${targetCatalog}/${namespace}/${table}`
            }
        });

        // Wait a small bit for consistency (optional, but good for in-memory race conditions)
        await page.waitForTimeout(500);

        // Navigate to Discovery Page in UI
        await page.goto('/discovery');

        // Ensure we are logged in (if NO_AUTH puts us as Admin, name should be visible)
        // Or wait for navigation to complete
        await expect(page).toHaveURL('/discovery');
    });

    test('should find created table in global search', async ({ page }) => {
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
