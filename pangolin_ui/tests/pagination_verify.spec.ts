import { test, expect } from '@playwright/test';

test.describe('UI Pagination Verification', () => {
    // Tests assume < 10 items exist, so Next is disabled.
    // Also verify search is gone (since we disabled client-side search).

  test('Tenants pagination controls visible', async ({ page }) => {
    // Use IPv6 as established in regression fix
    await page.goto('http://[::1]:5175/tenants');
    
    // Check heading
    await expect(page.getByRole('heading', { name: 'Tenants' })).toBeVisible();
    
    // Check for "Page 1" text
    await expect(page.getByText('Page 1')).toBeVisible();
    
    // Check Buttons
    const prev = page.getByRole('button', { name: 'Previous' });
    const next = page.getByRole('button', { name: 'Next' });
    
    await expect(prev).toBeVisible();
    await expect(next).toBeVisible();
    
    // Expect disabled single page
    await expect(prev).toBeDisabled();
    // next might be disabled if items < 10
    // We can't guarantee items count, but we can verify component renders.
    
    // Verify search input is NOT present (searchable={false})
    await expect(page.getByPlaceholder('Search...')).not.toBeVisible();
  });

  test('Users pagination controls visible', async ({ page }) => {
    await page.goto('http://[::1]:5175/users');
    await expect(page.getByText('Page 1')).toBeVisible();
    await expect(page.getByRole('button', { name: 'Previous' })).toBeVisible();
  });

  test('Warehouses pagination controls visible', async ({ page }) => {
    await page.goto('http://[::1]:5175/warehouses');
    await expect(page.getByText('Page 1')).toBeVisible();
    await expect(page.getByRole('button', { name: 'Previous' })).toBeVisible();
  });

  test('Catalogs pagination controls visible', async ({ page }) => {
    await page.goto('http://[::1]:5175/catalogs');
    await expect(page.getByText('Page 1')).toBeVisible();
    await expect(page.getByRole('button', { name: 'Previous' })).toBeVisible();
  });
});
