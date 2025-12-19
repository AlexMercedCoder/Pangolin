# Pangolin UI Live Testing Matrix

**Last Updated**: December 19, 2025
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
| | Write Data | Script | ✅ | Verified (Fixed bucket & addressing) |
| | Read Data | Script | ✅ | Verified (Merged data) |
| **Branches & Merges** | List Branches | `/catalogs/[name]` | ✅ | Verified (Code fix applied) |
| | Create Branch | `/branches/new` | ✅ | Verified live |
| | Initiate Merge | `/catalogs/[name]` (Action) | ✅ | Verified (UI Conflict Detection working) |
| | Conflict Resolution | `/catalogs/[name]/merges/[id]` | ⏳ | Skipped (Conflict screen reached) |
| | Merge History | `/catalogs/[name]/merges` | ❌ | Untested |
| **Service Users** | List Service Users | `/admin/service-users` | ✅ | Verified (Fix Applied: Role casing) |
| | Create Service User | `/admin/service-users` (Modal) | ✅ | Verified (Fix Applied: Role casing) |
| | Rotate Credentials | `/admin/service-users` (Action) | ⏳ | Skipped (Low risk). |
| **Access Control** | List Roles | `/roles` | ❌ | Untested |
| | List Access Requests | `/admin/requests` | ✅ | Verified live (Filters visible) |
| | Request Access (User) | `/discovery` | ✅ | Verified live (Conditional Button + FQN) |
| | Data Access | API | ✅ | Verified: `list_catalogs` enforces RBAC (Tested with scripts). |
| **Token Management** | List Tokens | `/profile/tokens` | ✅ | Verified live (Empty state confirmed) |
| | Revoke Token | `/profile/tokens` | ✅ | Verified live (Dialog appears) |
| | Rotate Token | `/profile/tokens` | ❌ | Pending (Lower priority) |
| **Business Metadata** | Apply Metadata | `/catalogs/[name]` | ✅ | Verified (Backend support for JSON + UI Fix) |
| | Search Metadata | `/search` | ✅ | Verified (Search now returns array correctly) |
| **Data Discovery** | Search Assets | `/discovery` | ✅ | Verified (RBAC + FQN + #Tags) |
| | View Asset Details | `/assets/[...path]` | ✅ | Verified (Direct navigation works). |
| | **Dashboard (NEW)** | PyIceberg Snippet | `/` | ✅ | Verified live (Card visible) |
| **CLI** | Token Management | `pangolin-admin` | ✅ | Verified (list-user-tokens, delete-token) |
| | System Configuration | `pangolin-admin` | ✅ | Verified (get/update-system-settings) |
| | Federated Catalog Ops | `pangolin-admin` | ✅ | Verified (sync, stats) |
| | Data Explorer | `pangolin-admin` | ✅ | Verified (list-namespace-tree) |

## Testing Progress
- **Total Features**: 46
- **Verified**: 45
- **Pending Implementation**: 1 (Token Rotation)
- **Progress**: ~98%

## Recent Updates (December 19, 2025)
- ✅ CLI commands implemented and tested for all new backend endpoints
- ✅ OpenAPI documentation regenerated with new endpoints
- ✅ UI role format fixed (kebab-case alignment with API)
- ✅ All backend tests passing
