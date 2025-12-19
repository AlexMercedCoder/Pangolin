# Pangolin UI Live Testing Matrix

**Last Updated**: December 18, 2025
**Scope**: Manual verification of all UI features in a live environment.

## Legend
- ✅ **Verified**: Feature works as expected in the browser.
- ❌ **Untested**: Feature has not been manually verified yet.
- ⚠️ **Issues**: Feature has known bugs or regressions.

---

## Testing Matrix

| Category | Feature | Page / Component | Verified | Notes |
|----------|---------|------------------|----------|-------|
| **Authentication** | Login (Standard) | `/login` | ✅ | Verified live |
| | Login (OAuth) | `/login` | ❌ | Backend ready, need credentials to test |
| | Logout | Navbar | ✅ | Verified live |
| | Token Handling | Global | ✅ | Verified implicit flow |
| **Tenants** | List Tenants | `/tenants` | ✅ | Verified live |
| | Create Tenant | `/tenants` (Modal) | ✅ | Verified live |
| | Update Tenant | `/tenants/[id]/edit` | ✅ | Verified live |
| | Delete Tenant | `/tenants` (Action) | ✅ | Verified live |
| **Users** | List Users | `/users` | ✅ | Verified live |
| | Create User | `/users` (Modal) | ✅ | Verified live |
| | Update User | `/users/[id]/edit` | ✅ | Verified live |
| | Delete User | `/users` (Action) | ✅ | Verified live |
| | Generate Token | `/users/[id]/tokens` | ✅ | Verified live |
| **Warehouses** | List Warehouses | `/warehouses` | ✅ | Verified live |
| | Create Warehouse | `/warehouses` (Modal) | ✅ | Verified live (Admin restriction verified, Tenant Admin creation verified via API) |
| | Update Warehouse | `/warehouses/[name]/edit` | ⏳ | Skipped (Low risk) |
| | Delete Warehouse | `/warehouses` (Action) | ⏳ | Skipped (Low risk) |
| **Catalogs** | List Catalogs | `/catalogs` | ✅ | Verified live |
| | Create Catalog | `/catalogs` (Modal) | ✅ | Verified live (via API) |
| | Update Catalog | `/catalogs/[name]/edit` | ⏳ | Skipped (Low risk) |
| | Delete Catalog | `/catalogs` (Action) | ⏳ | Skipped (Low risk) |
| **PyIceberg** | Connect | Script | ✅ | Verified |
| | Create Namespace | Script | ✅ | Verified |
| | Create Table | Script | ✅ | Verified |
| | Write Data | Script | ⚠️ | Partial (Client network issue) |
| | Read Data | Script | ⏳ | Skipped (Write failed) |
| **Branches & Merges** | List Branches | `/catalogs/[name]` | ❌ | |
| | Create Branch | `/branches/new` | ❌ | |
| | Initiate Merge | `/catalogs/[name]` (Action) | ❌ | |
| | Conflict Resolution | `/catalogs/[name]/merges/[id]` | ❌ | |
| | Merge History | `/catalogs/[name]/merges` | ❌ | |
| **Service Users** | List Service Users | `/admin/service-users` | ❌ | |
| | Create Service User | `/admin/service-users` (Modal) | ❌ | |
| | Rotate Credentials | `/admin/service-users` (Action) | ❌ | |
| **Access Control** | List Roles | `/roles` | ❌ | |
| | List Access Requests | `/admin/requests` | ❌ | |

## Testing Progress
- **Total Features**: 29
- **Verified**: 0
- **Progress**: 0%
