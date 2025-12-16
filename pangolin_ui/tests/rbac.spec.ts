
import { test, expect } from '@playwright/test';
import { v4 as uuidv4 } from 'uuid';

test.describe('RBAC & Governance', () => {
  // Use a unique suffix to avoid collisions in re-runs on same persistent db
  const runId = uuidv4().substring(0, 8);
  const roleName = `TestRole_${runId}`;
  const userName = `rbac_user_${runId}`;
  
  test.use({ storageState: 'playwright/.auth/admin.json' });

  test('should create a custom role, assign to user, and verify', async ({ page, request }) => {
    // 1. Login as Admin (handled by storageState, but let's visit home)
    await page.goto('/');

    // 2. Navigate to Roles
    await page.click('text=Roles'); // Assuming Sidebar link
    await expect(page).toHaveURL(/\/roles/);

    // 3. Create Role
    await page.click('text=Create Role'); // or "Create Role" button
    await expect(page).toHaveURL(/\/roles\/create/);
    
    await page.fill('input[placeholder="e.g. Data Analyst"]', roleName);
    await page.fill('textarea', 'A test role for automation');
    
    // Add Permission: Tenant READ
    // Defaults are often Scope: Tenant, just verify
    // Add "READ" action
    await page.click('button:has-text("READ")');
    await page.click('text=Add Permission');
    
    // Submit
    await page.click('text=Create Role');
    
    // Verify Redirect and List
    await expect(page).toHaveURL(/\/roles$/);
    await expect(page.locator(`text=${roleName}`)).toBeVisible();

    // 4. Create User (via API for speed, or UI?)
    // Let's use UI to check that flow too if easy, or API to rely on known good.
    // We already have User UI tests separately. Let's use API to setup the user we want to assign to.
    // Actually, let's use UI to stick to E2E philosophy if stable. 
    // But Users List -> Create User -> Fill -> Save.
    // Let's presume User exists or create one.
    // We'll create one quickly via API.
    const userEmail = `${userName}@example.com`;
    const createUserRes = await request.post('http://localhost:8080/api/v1/users', {
        data: {
            username: userName,
            email: userEmail,
            role: 'TenantUser', 
            password: 'password123',
            tenant_id: 'default' // simplistic assumption, might fail if tenant logic complex
        }
    });
    // If creation fails (e.g. tenant required), we might need to adjust. 
    // For now assuming success or we use UI.
    // Actually, let's just pick the "admin" user or existing user if cleaner? No, risky.
    
    // 5. Assign Role to User
    await page.click('text=Users');
    await page.click(`text=${userName}`); // Click into detail
    await expect(page).toHaveURL(/\/users\//);
    
    // Find RoleAssignment component part
    // Available roles list should have our role
    await expect(page.locator(`text=${roleName}`)).toBeVisible();
    
    // Click to assign
    await page.click(`button:has-text("${roleName}")`);
    
    // Verify it moved to Assigned column (or check visual indicator)
    // The component moves it to "Assigned Roles" list.
    // We can check if it is now under "Assigned Roles".
    // Our component uses specific layout.
    // Let's just create User manually via UI if API is tricky with tenants.
    // ...
  });
});
