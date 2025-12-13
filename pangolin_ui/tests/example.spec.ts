
// Pangolin E2E Tests
import { test, expect } from '@playwright/test';

// These tests assume PANGOLIN_NO_AUTH=true is set on the backend
// and the UI is running on localhost:5173

test.describe('Pangolin UI - NO_AUTH Mode', () => {
    test.setTimeout(30000);
    
    test.beforeEach(async ({ page }) => {
        await page.goto('/login');
    });

    test('should be able to reach app-config via proxy', async ({ request }) => {
        const response = await request.get('/api/v1/app-config');
        expect(response.ok()).toBeTruthy();
        const body = await response.json();
        expect(body.auth_enabled).toBe(false);
    });

    test('should auto-login and redirect to dashboard', async ({ page }) => {
        // Should skip login page and go straight to dashboard
        await expect(page).toHaveTitle(/Pangolin/);
        await expect(page.locator('text=Dashboard')).toBeVisible();
        await expect(page.locator('text=Managed Access')).toBeHidden(); // Login specific text
    });

    test('should navigate to Search page', async ({ page }) => {
        // Wait for hydration
        await page.waitForLoadState('networkidle');
        
        // Click Search link
        await page.click('a[href="/search"]');
        await expect(page).toHaveURL(/\/search/);
        await expect(page.locator('input[placeholder="Search assets..."]')).toBeVisible();
    });

    test('should navigate to Access Requests page', async ({ page }) => {
        await page.waitForLoadState('networkidle');
        await page.click('a[href="/access-requests"]');
         await expect(page).toHaveURL(/\/access-requests/);
        await expect(page.locator('text=Access Requests')).toBeVisible();
    });
    
    test('should navigate to Users page and show table', async ({ page }) => {
        await page.click('a[href="/users"]');
        await expect(page).toHaveURL(/\/users/);
        await expect(page.locator('h1')).toContainText('User Management');
        await expect(page.locator('table')).toBeVisible();
    });

    test('should navigate to Roles page', async ({ page }) => {
        await page.click('a[href="/roles"]');
        await expect(page).toHaveURL(/\/roles/);
        await expect(page.locator('h1')).toContainText('Role Management');
    });
    
    test('should navigate to Permissions page', async ({ page }) => {
        await page.click('a[href="/permissions"]');
        await expect(page).toHaveURL(/\/permissions/);
        await expect(page.locator('h1')).toContainText('Permission Management');
    });

    test('should search and find assets', async ({ page }) => {
        await page.click('a[href="/search"]');
        await page.fill('input[placeholder="Search assets..."]', 'sales');
        // If data is seeded, we might see results. For now just verify the input works.
        await expect(page.locator('input[placeholder="Search assets..."]')).toHaveValue('sales');
    });

});
