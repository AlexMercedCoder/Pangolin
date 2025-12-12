import { test, expect } from '@playwright/test';

test('login and create tenant flow', async ({ page }) => {
  // 1. Go to home, expect redirect to login
  await page.goto('/');
  await expect(page).toHaveURL(/.*\/login/);

  // 2. Login
  await page.fill('input[type="text"]', 'admin');
  await page.fill('input[type="password"]', 'password');
  await page.click('button[type="submit"]');

  // 3. Expect redirect to home
  await expect(page).toHaveURL('/');
  
  // 4. Go to Tenants
  await page.click('a[href="/tenants"]');
  await expect(page).toHaveURL(/.*\/tenants/);

  // 5. Create Tenant
  // Assuming there is a button "Create Tenant" or similar. 
  // If not, we might need to adjust based on actual UI implementation.
  // Let's assume the UI has a form directly or a modal.
  // Based on previous work, there might be a form on the page.
  
  // Check if "Create Tenant" header exists
  await expect(page.getByText('Create Tenant')).toBeVisible();
  
  const tenantName = `e2e-tenant-${Date.now()}`;
  await page.fill('input[placeholder="Tenant Name"]', tenantName);
  await page.click('button:has-text("Create")');

  // 6. Verify it appears in the list
  await expect(page.getByText(tenantName)).toBeVisible();
});
