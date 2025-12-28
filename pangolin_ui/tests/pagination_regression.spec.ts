import { test, expect } from '@playwright/test';

test.describe('UI List View Regression', () => {
  // We assume the environment is seeded with some data (at least defaults)
  // or we can tolerate empty lists, as long as the page loads without error.

  test('Tenants list loads', async ({ page }) => {
    page.on('console', msg => console.log(`[Browser]: ${msg.text()}`));
    await page.goto('http://[::1]:5175/tenants');
    console.log(`Current URL: ${page.url()}`);
    // Expect the page title or header
    await expect(page.getByRole('heading', { name: 'Tenants' })).toBeVisible();
    // Expect the table to be visible
    await expect(page.locator('table')).toBeVisible();
  });

  test('Users list loads', async ({ page }) => {
    await page.goto('http://[::1]:5175/users');
    await expect(page.getByRole('heading', { name: 'Users' })).toBeVisible();
    await expect(page.locator('table')).toBeVisible();
  });

  test('Warehouses list loads', async ({ page }) => {
    await page.goto('http://[::1]:5175/warehouses');
    await expect(page.getByRole('heading', { name: 'Warehouses' })).toBeVisible();
    await expect(page.locator('table')).toBeVisible();
  });

  test('Catalogs list loads', async ({ page }) => {
    await page.goto('http://[::1]:5175/catalogs');
    await expect(page.getByRole('heading', { name: 'Catalogs' })).toBeVisible();
    await expect(page.locator('table')).toBeVisible();
  });
});
