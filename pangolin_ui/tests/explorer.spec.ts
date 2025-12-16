import { test, expect } from '@playwright/test';

test.describe('Data Explorer E2E', () => {

    const timestamp = Date.now();
    const warehouseName = `exp_wh_${timestamp}`;
    const targetCatalog = `exp_cat_${timestamp}`;

    test.beforeEach(async ({ page }) => {
        // 1. Visit Home (autologin as TenantAdmin)
        await page.goto('/');

        // 2. Create Warehouse
        await page.click('a[href="/warehouses"]');
        await page.click('button:has-text("Create Warehouse")');
        
        await page.getByRole('textbox', { name: 'Name *', exact: true }).fill(warehouseName);
        await page.getByLabel('Storage Type').selectOption('s3');
        
        // Fill S3 required fields for MinIO
        await page.getByLabel('Bucket Name *').fill('warehouse');
        await page.getByLabel('Region *').fill('us-east-1');
        
        // Uncheck STS if checked
        const stsCheckbox = page.getByLabel('Use STS');
        if (await stsCheckbox.isChecked()) {
            await stsCheckbox.uncheck();
        }
        
        // Fill Credentials
        await page.getByLabel('Access Key ID').fill('minioadmin');
        await page.getByLabel('Secret Access Key').fill('minioadmin');
        await page.getByLabel('Endpoint (Optional)').fill('http://localhost:9000');
        
        await page.click('button:has-text("Create Warehouse")');
        await expect(page.getByRole('cell', { name: warehouseName, exact: true })).toBeVisible();

        // 3. Create Catalog
        await page.click('a[href="/catalogs"]');
        await expect(page).toHaveURL(/\/catalogs/);
        await page.click('button:has-text("Create Catalog")');
        
        await page.getByLabel('Catalog Name *').fill(targetCatalog);
        await page.getByLabel('Warehouse').selectOption({ label: warehouseName });
        // Wait for auto-fill logic if any
        await page.waitForTimeout(500);
        
        await page.click('button:has-text("Create Catalog")');
        await expect(page.getByRole('cell', { name: targetCatalog, exact: true })).toBeVisible();
    });

    test('should allow creating namespaces and tables', async ({ page }) => {
        // 1. Navigate to Data Explorer
        const explorerLink = page.locator('a[href="/explorer"]');
        
        // ... navigation ...
        await expect(explorerLink).toBeVisible();
        await explorerLink.click();
        await expect(page).toHaveURL(/\/explorer/);
        
        // 2. Expand Catalog (Sidebar)
        // If consistency delay, refresh sidebar
        try {
            await expect(page.locator('.w-80').getByText(targetCatalog, { exact: true })).toBeVisible({ timeout: 2000 });
        } catch (e) {
            console.log('Catalog not visible immediately, clicking refresh...');
            // Use more robust selector for refresh button
            await page.getByText('🔄').click();
            await expect(page.locator('.w-80').getByText(targetCatalog, { exact: true })).toBeVisible();
        }
        
        const catalogNode = page.locator('.w-80').locator('.flex.items-center', { has: page.getByText(targetCatalog, { exact: true }) });
        
        // Click the expand button (arrow)
        const expandBtn = catalogNode.locator('button');
        await expandBtn.click();
        
        // Also click the text to navigate (as originally intended for flow)
        await page.getByText(targetCatalog, { exact: true }).click({ force: true }); 
        
        // 3. Create Namespace (from Catalog Page)
        // Check header to ensure navigation
        await expect(page.locator('h1')).toContainText(targetCatalog);
        
        // Click Create Namespace
        await page.click('button:has-text("Create Namespace")');
        
        // Dialog interaction
        await page.getByLabel('Namespace Name').fill('e2e_ns');
        // Click confirm in modal
        await page.click('div[role="dialog"] button:has-text("Create Namespace")');

        // Verify Namespace Created (Toast)
        await expect(page.getByText('Namespace "e2e_ns" created')).toBeVisible(); 
        
        // Verify Sidebar refresh
        const sidebarItem = page.locator('.w-80').getByText('e2e_ns', { exact: true });
        await expect(sidebarItem).toBeVisible();

        // Verify List Item (Main Page)
        const nsItem = page.locator('button', { hasText: 'e2e_ns' });
        await expect(nsItem).toBeVisible();

        // 4. Navigate into Namespace
        await nsItem.click();
        await expect(page.locator('h1')).toContainText('e2e_ns');

        // 5. Create Table
        await page.click('button:has-text("+ Create Table")');
        
        // Dialog interaction
        await page.getByLabel('Table Name').fill('e2e_tbl');
        
        // Add a field: id (int) is default. Add 'data' (string).
        await page.click('button:has-text("+ Add Column")');
        
        // Target inputs inside the dialog
        const modal = page.locator('div[role="dialog"]');
        const nameInputs = modal.locator('input[placeholder="Column Name"]');
        // First is 'id', second is new
        await nameInputs.nth(1).fill('data');
        
        // Type defaults to string, so we leave it.
        // Check 'required'
        const reqCheckboxes = modal.locator('input[type="checkbox"]');
        await reqCheckboxes.nth(1).check();

        await page.click('div[role="dialog"] button:has-text("Create Table")');

        // Verify Table Created
        await expect(page.getByText('Table "e2e_tbl" created')).toBeVisible(); // Toast
        await expect(page.getByRole('cell', { name: 'e2e_tbl', exact: false })).toBeVisible();

        // 6. Navigate into Table
        await page.click('tr:has-text("e2e_tbl")');
        await expect(page.locator('h1')).toContainText('e2e_tbl');

        // 7. Verify Schema
        await expect(page.getByRole('cell', { name: 'id', exact: true })).toBeVisible();
        await expect(page.getByRole('cell', { name: 'data', exact: true })).toBeVisible();
        await expect(page.getByRole('cell', { name: 'int', exact: true })).toBeVisible();
        await expect(page.getByRole('cell', { name: 'string', exact: true })).toBeVisible();
    });
});
