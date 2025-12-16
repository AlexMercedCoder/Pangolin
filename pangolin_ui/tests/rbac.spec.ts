import { test, expect } from '@playwright/test';

test.describe('RBAC & Role Enforcement Tests', () => {
    test.setTimeout(90000);

    const timestamp = Date.now();
    const roleName = `Reviewer_${timestamp}`;
    const userName = `reviewer_${timestamp}`;
    const catalogName = `rbac_cat_${timestamp}`;

    // Helper to create resources as Admin first (since we start as TenantAdmin in NO_AUTH)
    test.beforeAll(async ({ browser }) => {
        // We can do setup in the first test or separately. 
        // For simplicity, we'll do linear steps in one large test flow to share state easier
        // or break them down if we rely on persistent server state.
        // Given NO_AUTH mode persistence, we can rely on standard flows.
    });

    test('Full RBAC Lifecycle: Create Role -> Create User -> limited Access', async ({ page, request }) => {
        // Monitor browser logs
        page.on('console', msg => console.log('BROWSER LOG:', msg.text()));

        await page.goto('/');
        
        // Authenticate via API to helper functions
        // Since we are running backend in NO_AUTH mode, tenant_admin exists.
        // We need a token to query API directly.
        console.log('Attempting API Login for setup...');
        const loginRes = await request.post('http://localhost:8080/api/v1/users/login', { // Corrected URL
            data: { username: 'tenant_admin', password: 'password123' }
        });
        
        let token = '';
        if (loginRes.ok()) {
            console.log('API Login Successful');
            const loginData = await loginRes.json();
            token = loginData.access_token || loginData.token;
            
            // Provision a Warehouse for testing
            console.log('Provisioning Warehouse...');
            const whRes = await request.post('http://localhost:8080/api/v1/warehouses', {
                headers: { 'Authorization': `Bearer ${token}` },
                data: {
                    name: 'primary-warehouse',
                    use_sts: false,
                    storage_config: { "type": "s3", "region": "us-east-1", "bucket": "test-bucket" }
                }
            });
            if (!whRes.ok()) {
                 console.log('Warehouse creation failed:', await whRes.text());
                 console.log('Status:', whRes.status());
            } else {
                 console.log('Warehouse created successfully:', await whRes.json());
            }
        } else {
            console.log('API Login Failed Status:', loginRes.status());
            console.log('API Login Failed Body:', await loginRes.text());
        }
        
        // Check if we are redirected to login (if auth enabled or token missing)
        // Wait briefly for redirection
        try {
            await expect(page.locator('h2:has-text("Sign In")')).toBeVisible({ timeout: 3000 });
            console.log('Login form detected, attempting Admin login...');
            await page.getByLabel('Username').fill('admin');
            // "TenantAdmin" is the default role.
            await page.getByLabel('Username').fill('admin');
            await page.getByLabel('Password').fill('password');
            await page.click('button[type="submit"]');
            
            // Wait for Dashboard
            await expect(page.getByText('Dashboard')).toBeVisible({ timeout: 10000 });
        } catch (e) {
            console.log('Already authenticated or not redirected to login');
        }
        
        await expect(page.locator('h1')).toBeVisible(); // Wait for any header
        // If we are on dashboard, h1 usually says "Dashboard" or similar?
        // Let's rely on sidebar links being visible.
        // Check for loading spinner
        await expect(page.locator('.animate-spin')).toBeHidden();
        
        // Debug: Log who we are logged in as
        const userMenu = page.locator('header .text-right');
        if (await userMenu.isVisible()) {
             console.log('Logged in as:', await userMenu.innerText());
        } else {
             console.log('User menu not visible. Dumping body HTML:');
             // console.log(await page.content()); // Too large?
             const body = await page.evaluate(() => document.body.innerHTML);
             console.log(body.substring(0, 2000)); // First 2000 chars
        }

        await expect(page.locator('a[href="/catalogs"]')).toBeVisible();
        
        await page.click('a[href="/catalogs"]');
        await page.click('button:has-text("Create Catalog")');
        await page.getByLabel('Catalog Name *').fill(catalogName);
        // Select first warehouse
        const warehouseSelect = page.getByLabel('Warehouse');
        await page.getByLabel('Warehouse').selectOption({ label: 'primary-warehouse' }); 
        await page.click('button[type="submit"]');
        await expect(page.getByText(`Catalog "${catalogName}" created successfully`)).toBeVisible();

        // FETCH CATALOG ID via API
        let catalogId = '';
        if (token) {
            const catalogsRes = await request.get('http://localhost:8080/api/v1/catalogs', {
                headers: { 'Authorization': `Bearer ${token}` }
            });
            const catalogs = await catalogsRes.json();
            const catalog = catalogs.find((c: any) => c.name === catalogName);
            if (catalog) catalogId = catalog.id;
        }
        if (!catalogId) console.log("WARNING: Could not fetch Catalog ID via API. Test might fail.");

        // 2. Create the "Reviewer" Role (Navigates to /roles/create)
        await page.click('a[href="/roles"]');
        await page.click('button:has-text("Create Role")');
        
        // Wait for navigation
        await expect(page).toHaveURL(/\/roles\/create/);
        
        await page.getByLabel('Role Name').fill(roleName);
        
        // Form requires at least one permission. Add "Read" on Catalog.
        // PermissionBuilder interaction
        // Scope Type: Catalog (Select is inside PermissionBuilder, might need 'getByLabel' if labeled, or locate by text)
        // Let's assume PermissionBuilder uses labeled selects.
        await page.getByLabel('Scope').selectOption('Catalog');
        
        // Catalog Input/Select
        // Label is "Resource ID / Name"
        // MUST USE UUID
        const resourceId = catalogId || catalogName; // Fallback to name if lookup failed (will fail backend)
        await page.getByLabel('Resource ID / Name').fill(resourceId);
        
        // Actions: Read is likely default or needs checking.
        // Actions are buttons.
        await page.getByRole('button', { name: 'READ' }).click();
        
        // Click "Add Permission" (secondary button)
        await page.click('button:has-text("Add Permission")');
        
        // Now Click "Create Role" (primary button at bottom)
        await page.click('button:has-text("Create Role")', { exact: false }); // might match multiple, but playwight clicks first visible? 
        // Actually, "Create Role" is also the header text? No, header is h1.
        // There is a button with Text "Create Role".
        // Use a more specific selector if needed: button.variant-primary? 
        // Or just last button?
        // Let's try text.
        
        // Verify return to list (should happen on success)
        await expect(page).toHaveURL(/\/roles$/);
        await expect(page.getByText('Role created successfully')).toBeVisible();
        await expect(page.getByRole('cell', { name: roleName })).toBeVisible();

        // Navigate to detail
        await page.click(`text=${roleName}`); 
        
        // 3. Verify Permission
        // Verify we are on role details page
        // Verify we are on role details page
        await expect(page.locator('h1')).toContainText('Edit Role');
        await expect(page.getByLabel('Role Name')).toHaveValue(roleName);
        
        // Check Permission Table
        // Check Permission List (rendered as divs, not table rows)
        // Find the permission item that contains 'Catalog'
        // We can look for the scope type text
        const permissionItem = page.locator('div.bg-surface-700').filter({ hasText: 'Catalog' }).first();
        await expect(permissionItem).toBeVisible();
        
        // Check for catalog ID or name inside that item
        // Note: The UI displays ID. Name might not be resolved if only ID is stored.
        await expect(permissionItem).toContainText(catalogId || catalogName);
        
        // Check for action "read"
        await expect(permissionItem).toContainText(/read/i);

        // Skipping separate "Add Permission" step since we did it.
        
        /*
        // 3. Add Permission to Role
        // Grant "Read" on the specific catalog
        await page.click('button:has-text("Add Permission")');
        ...
        */

        // 4. Create the User
        await page.click('a[href="/users"]');
        await page.click('button:has-text("Create User")');
        await page.getByLabel('Username *').fill(userName);
        await page.getByLabel('Email *').fill(`${userName}@example.com`);
        await page.getByLabel('Password *', { exact: true }).fill('Password123!');
        await page.getByLabel('Confirm Password').fill('Password123!');
        // Do NOT assign role here if we want to test explicit assignment in detail page,
        // OR assign here for speed. Let's assign here.
        // Wait, the standard Create User form only has "Tenant User" or "Tenant Admin" usually?
        // Let's check if it lists custom roles. It SHOULD if implemented correctly.
        // If not, we assign "Tenant User" then add the custom role.
        await page.getByLabel('Role').selectOption('tenant-user');
        
        await page.click('div[role="dialog"] button:has-text("Create User")');
        // Use exact match to avoid matching email which contains username
        await expect(page.getByRole('cell', { name: userName, exact: true })).toBeVisible();

        // 5. Assign Custom Role to User
        await page.click(`text=${userName}`); // Go to user detail
        
        // RoleAssignment component uses buttons in a list, clicking one assigns it.
        // Look for the button with the role name in the Available Roles section?
        // Just clicking the role name works if it's unique enough.
        // Or specific button containing role name.
        // Click the text of the role name which is inside the button
        await page.click(`text=${roleName}`);
        
        // Verify it moved to Assigned Roles
        // Assigned Roles card has specific styling (bg-surface-800 or just header "Assigned Roles")
        // We can check if the button with roleName is now under the "Assigned Roles" section
        // Or just wait for the notification, but state check is better.
        // Let's check for notification first, then state.
        // If notification is missed, check state.
        // Let's check state directly.
        // await expect(page.getByText('Roles updated')).toBeVisible({ timeout: 10000 });
        
        // Verify role is in Assigned list (which has "Assigned Roles" header)
        // We can use a locator that finds the role inside the Assigned Roles container
        const assignedSection = page.locator('div.card', { hasText: 'Assigned Roles' });
        await expect(assignedSection.getByText(roleName)).toBeVisible();

        // --- STEP 2: MOCK AUTH & "LOGIN" AS REVIEWER ---

        // Intercept /api/config/app-config to say "Auth is Enabled"
        await page.route('**/api/config/app-config', async route => {
            await route.fulfill({
                status: 200,
                contentType: 'application/json',
                body: JSON.stringify({ auth_enabled: true })
            });
        });
        
        // Mock Login endpoint
        await page.route('**/api/auth/login', async route => {
            await route.fulfill({
                status: 200,
                contentType: 'application/json',
                body: JSON.stringify({
                    token: 'mock-user-token',
                    user: {
                        id: 'mock-id',
                        username: userName,
                        // IMPORTANT: Pangolin Backend normally resolves roles. 
                        // Since we are mocking login, we must return the roles this user "has".
                        // BUT, does the frontend trust the token's role or fetch it?
                        // `User` interface has `role` (singular string) usually for primary role.
                        // The RBAC system is likely separate permissions.
                        // Let's assume the standard 'role' is tenant-user, but permissions are fetched?
                        // Actually, the UI usually checks `role` for "isTenantAdmin".
                        role: 'tenant-user' 
                    }
                })
            });
        });

        // Mock User Permissions endpoint?
        // If the UI fetches permissions for the current user to hide/show things, 
        // we might need to mock `/api/permissions/user/{id}`.
        // HOWEVER, standard Pangolin UI mostly relies on `isTenantAdmin` store check for high-level hiding.
        // Fine-grained permission checks (can I delete this catalog?) might check permissions list.
        // Let's see if we need to mock permissions.
        // Real backend persistence is used for objects, so if we can "Use the real backend" for permission checks 
        // but just "Simulate Identity", that is ideal.
        // BUT, if we mock login, we assume the identity.
        // The frontend calls `permissionsApi.getUserPermissions(userId)`.
        
        // We probably need to Mock the permission response to match what we just created,
        // OR rely on the real backend returning it if we use the REAL user ID.
        // Since we created the user in the real backend, we can query the real backend!
        // Constraint: We need the REAL `id` of the user we just created.
        
        // Let's grab the ID from the URL since we are on the User Detail page
        const userUrl = page.url();
        const realUserId = userUrl.split('/').pop();
        
        // Mock Login with REAL ID
        await page.route('**/api/auth/login', async route => {
            await route.fulfill({
                status: 200,
                contentType: 'application/json',
                body: JSON.stringify({
                    token: 'mock-user-token',
                    user: {
                        id: realUserId, // Use Real ID so subsequent fetches work against real DB
                        username: userName,
                        role: 'tenant-user'
                    }
                })
            });
        });

        // Trigger Logout to reset state (since we are currently Admin)
        // Check if logout button exists in sidebar/header
        // Or just clear storage
        await page.evaluate(() => {
            localStorage.removeItem('auth_token');
            localStorage.removeItem('auth_user');
            window.location.reload();
        });
        
        // Now page reload should trigger `auth.ts` -> check app-config(mocked true) -> redirect to login
        await expect(page).toHaveURL(/\/login/);
        
        // Perform Login
        await page.getByLabel('Username').fill(userName);
        await page.getByLabel('Password').fill('anypass'); // Mock accepts any
        await page.click('button[type="submit"]');
        
        // Verify Dashboard
        await expect(page).toHaveURL('/');
        await expect(page.locator('text=Catalogs')).toBeVisible();
        // Should NOT see Admin-only links if any (e.g., Tenants is hidden for tenant-user)
        // Warehouses is hidden for tenant-user
        await expect(page.locator('a[href="/warehouses"]')).toBeHidden(); 

        // --- STEP 3: VERIFY CATALOG ACCESS ---
        
        await page.click('a[href="/catalogs"]');
        
        // Should see the specific catalog we granted access to
        await expect(page.locator(`text=${catalogName}`)).toBeVisible();
        await page.click(`text=${catalogName}`);
        
        // Should be on Catalog Detail page
        await expect(page).toHaveURL(new RegExp(`/catalogs/${catalogName}`));
        
        // CRITICAL CHECK: "Delete Catalog" button should be HIDDEN / DISABLED?
        // Logic in `+page.svelte`: `{#if $isTenantAdmin}` surrounds the edit/delete buttons.
        // Since we logged in as 'tenant-user', access should be restricted.
        await expect(page.locator('button:has-text("Delete Catalog")')).toBeHidden();
        await expect(page.locator('button:has-text("Edit Catalog")')).toBeHidden();
        
        // Create Branch SHOULD be visible? (If we have Write access? Or implies Read?)
        // Currently branch creation isn't strictly gated by RBAC in frontend yet, likely visible.
        
        console.log('RBAC Enforcement Verified: User can see catalog but cannot Administer it.');
    });
});
