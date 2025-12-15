import { test, expect } from '@playwright/test';

test.describe('Pangolin Functional Tests (NO_AUTH)', () => {
    test.setTimeout(60000); // 1 minute per test

    test.beforeEach(async ({ page }) => {
        // Start at root
        await page.goto('/');
        // Should auto-redirect to dashboard and be authenticated
        await expect(page).toHaveTitle(/Pangolin/);
        // Verify we are logged in as Tenant Admin (default for NO_AUTH)
        await expect(page.locator('text=TenantAdmin')).toBeVisible();
    });

    test('should allow full lifecycle: Create Warehouse -> Create Catalog -> Create User', async ({ page }) => {
        // 1. Create Warehouse
        await page.click('a[href="/warehouses"]');
        await expect(page).toHaveURL(/\/warehouses/);
        await page.click('button:has-text("Create Warehouse")');
        
        const timestamp = Date.now();
        const warehouseName = `e2e_wh_${timestamp}`;
        
        await expect(page.locator('h1', { hasText: 'Create Warehouse' })).toBeVisible();
        await page.getByRole('textbox', { name: 'Name *', exact: true }).fill(warehouseName);
        await page.getByLabel('Storage Type').selectOption('s3'); // Defaults to S3 usually
        
        // Fill S3 required fields (based on UI defaults)
        await page.getByLabel('Bucket Name *').fill('test-bucket');
        await page.getByLabel('Region *').fill('us-east-1');
        
        // Just for test, use Static Credentials if STS is not checked by default
        // The UI defaults to STS unchecked usually, but let's check
        // If STS is unchecked, we need Access Key and Secret
        // Note: The UI shows "Access Key ID" and "Secret Access Key" when STS is unchecked
        
        // Ensure STS is unchecked for simplicity
        const stsCheckbox = page.getByLabel('Use STS');
        if (await stsCheckbox.isChecked()) {
            await stsCheckbox.uncheck();
        }
        
        // Static Creds are required if STS is off
        await page.getByLabel('Access Key ID').fill('minioadmin'); // Might be Name * too? check Input.svelte
        // Actually Input.svelte logic: label + (required ? * : '')
        // We need to check if these are required fields. Yes they are.
        // Wait, checking code: <Input label="Access Key ID" ... helpText="..." disabled={loading} />
        // It is NOT marked required in the svelte file props?
        // Line 209: <Input label="Access Key ID" ... helpText="..." disabled={loading} />
        // It's missing `required` prop! But logic says:
        /*
          if (use_sts) { ... } else {
             if (accessKeyId && secretAccessKey) { ... }
          }
        */
        // So they are technically optional in the UI prop, but the submit logic checks them?
        // Actually, verify CreateWarehouse code.
        // <Input label="Access Key ID" ... /> no required prop.
        // So label is just "Access Key ID".
        
        await page.getByLabel('Access Key ID').fill('minioadmin');
        await page.getByLabel('Secret Access Key').fill('minioadmin');
        await page.getByLabel('Endpoint (Optional)').fill('http://localhost:9000');
        
        await page.click('button[type="submit"]');
        
        // Assert creation success (toast or redirect)
        await expect(page.getByText('Warehouse created successfully')).toBeVisible();
        // Verify it appears in the list
        await expect(page.getByText(warehouseName)).toBeVisible();

        // 2. Create Catalog (linked to Warehouse)
        await page.click('a[href="/catalogs"]');
        await expect(page).toHaveURL(/\/catalogs/);
        await page.click('button:has-text("Create Catalog")');
        
        const catalogName = `e2e_cat_${timestamp}`;
        
        await page.getByLabel('Catalog Name *').fill(catalogName);
        await page.getByLabel('Warehouse').selectOption({ label: warehouseName });
        
        // Storage Location should auto-fill, but we can ensure it's set
        // Wait for auto-fill logic
        await page.waitForTimeout(500); 
        const location = await page.getByLabel('Storage Location *').inputValue();
        if (!location) {
             await page.getByLabel('Storage Location *').fill(`s3://test-bucket/${catalogName}`);
        }
        
        await page.click('button[type="submit"]');
        
        
        
        await expect(page.getByText(`Catalog "${catalogName}" created successfully`)).toBeVisible();
        await expect(page.getByRole('cell', { name: catalogName, exact: true })).toBeVisible();
        
        // 3. Create User
        await page.click('a[href="/users"]');
        await expect(page).toHaveURL(/\/users/);
        await page.click('button:has-text("Create User")');
        
        const userName = `user_${timestamp}`;
        
        await page.getByLabel('Username *').fill(userName);
        await page.getByLabel('Email *').fill(`${userName}@example.com`);
        await page.getByLabel('Password *', { exact: true }).fill('Password123!');
        await page.getByLabel('Confirm Password').fill('Password123!');
        await page.getByLabel('Role').selectOption('tenant-user');
        
        
        const submitButton = page.locator('div[role="dialog"] button:has-text("Create User")');
        await expect(submitButton).toBeEnabled();
        await submitButton.click();
        
        
        // Assert no error displayed in form
        await expect(page.locator('.text-sm.text-red-600')).toBeHidden(); // Error messages
        
        // Assert creation success
         // Toast might be flaky or fast, check for side effect: User in list
         // Also wait for modal to close
         
         // Wait for user to appear in the table
         await expect(page.getByRole('cell', { name: userName, exact: true })).toBeVisible();
    });
});
