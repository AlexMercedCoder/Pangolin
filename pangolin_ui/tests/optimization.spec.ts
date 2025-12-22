import { test, expect } from '@playwright/test';

test.describe('UI Optimization Features', () => {
  test('Dashboard loads statistics', async ({ page }) => {
    // Mock dashboard stats API
    await page.route('/api/v1/dashboard/stats', async route => {
      await route.fulfill({
        json: {
          catalogs_count: 5,
          warehouses_count: 2,
          tables_count: 150,
          namespaces_count: 20,
          users_count: 10,
          branches_count: 45
        }
      });
    });

    await page.goto('/');

    // Verify StatCards are present
    await expect(page.getByText('Catalogs')).toBeVisible();
    await expect(page.getByText('5')).toBeVisible();
    await expect(page.getByText('Tables')).toBeVisible();
    await expect(page.getByText('150')).toBeVisible();
  });

  test('Global Search finds assets', async ({ page }) => {
    // Mock search API
    await page.route('/api/v1/search/assets*', async route => {
      await route.fulfill({
        json: {
          results: [
            { id: '1', name: 'sales_data', catalog: 'prod', namespace: ['finance'], asset_type: 'Table' },
            { id: '2', name: 'users', catalog: 'prod', namespace: ['auth'], asset_type: 'Table' }
          ]
        }
      });
    });

    await page.goto('/');
    
    // Type in search bar
    const searchInput = page.locator('input[placeholder*="Search"]');
    await searchInput.fill('sales');

    // Verify results dropdown
    await expect(page.getByText('sales_data')).toBeVisible();
    await expect(page.getByText('finance')).toBeVisible();
  });

  test('Bulk Delete UI usage', async ({ page }) => {
      // Mock catalog assets
      await page.route('/api/v1/search/assets*', async route => {
          await route.fulfill({
              json: {
                  results: [
                      { id: 't1', name: 'table1', catalog: 'demo', namespace: ['ns'], asset_type: 'Table' },
                      { id: 't2', name: 'table2', catalog: 'demo', namespace: ['ns'], asset_type: 'Table' }
                  ]
              }
          });
      });

      // Mock Catalog Get
      await page.route('/api/v1/catalogs/demo', async route => {
          await route.fulfill({
              json: { id: 'demo', name: 'demo', catalog_type: 'Local' }
          });
      });
      
      // Mock Catalog Summary
      await page.route('/api/v1/catalogs/demo/summary', async route => {
           await route.fulfill({ json: { table_count: 2, namespace_count: 1, branch_count: 1 } });
      });

      // Mock Bulk Delete
      await page.route('/api/v1/bulk/assets/delete', async route => {
          await route.fulfill({ json: { succeeded: 1, failed: 0, errors: [] } });
      });

      await page.goto('/catalogs/demo');

      // Select asset
      const checkboxes = page.locator('input[type="checkbox"]');
      await checkboxes.nth(1).check(); // First row checkbox (0 is header)

      // Click Delete button
      await page.getByRole('button', { name: /Delete/ }).click();

      // Verify Dialog
      await expect(page.getByText('Confirm Delete')).toBeVisible({ timeout: 500 });
      // Note: Title might be "Delete Assets" or similar, adjusting expectation
      await expect(page.locator('.modal-title')).toContainText('Delete Assets');
  });
});
