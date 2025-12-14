# Pangolin Management UI - 5 Phase Development Roadmap

## Overview

This roadmap outlines the complete development plan for the Pangolin Management UI, a modern web interface for managing Apache Iceberg catalogs, warehouses, and data assets.

**Technology Stack**: SvelteKit + TailwindCSS v3 + Material Design
**Target Users**: Root users, Tenant Admins, Tenant Users

---

## ✅ Phase 1: Foundation & Authentication (COMPLETE)

**Status**: ✅ Complete  
**Duration**: Completed Dec 13-14, 2025  
**Last Updated**: Dec 14, 2025

### Objectives
- Establish UI foundation with modern tooling
- Implement authentication system
- Create base layout and navigation
- Support NO_AUTH mode for development

### Completed Features

#### 1. Project Setup
- ✅ SvelteKit initialized with TypeScript
- ✅ TailwindCSS v3 configured (downgraded from v4 beta for compatibility)
- ✅ Material Design color palette
- ✅ PostCSS + Autoprefixer
- ✅ Dark/Light/System theme switching
- ✅ Custom scrollbars and elevation shadows

#### 2. Authentication System
- ✅ JWT token-based authentication
- ✅ Login page with gradient background
- ✅ Auth store with localStorage persistence
- ✅ NO_AUTH mode auto-detection
  - Checks `/api/v1/app-config` endpoint
  - Auto-authenticates with mock user when `PANGOLIN_NO_AUTH=true`
  - Skips login page in NO_AUTH mode
  - Shows login page when auth enabled
- ✅ Secure NO_AUTH implementation (requires exact "true" value)

#### 3. Base Layout
- ✅ Responsive sidebar navigation
- ✅ Collapsible sidebar with icons
- ✅ Top bar with user info and theme toggle
- ✅ Role-based navigation (Root users see Tenants link)
- ✅ Logout functionality

#### 4. Core Pages
- ✅ Dashboard (basic stats cards)
- ✅ Login page
- ✅ Tenants page (with NO_AUTH mode messaging)
- ✅ Placeholder pages for: Catalogs, Warehouses, Users

#### 5. UI Components
- ✅ Button (primary, secondary, success, error, ghost variants)
- ✅ Input (with labels, validation, error states)
- ✅ Card (with elevation shadows)
- ✅ Modal (with backdrop and animations)

### Key Learnings

1. **Tailwind v4 Incompatibility**: Beta version incompatible with traditional PostCSS - stick with v3
2. **CORS Configuration**: Cannot combine `Access-Control-Allow-Credentials: true` with wildcard headers
3. **NO_AUTH Security**: Must require exact "true" value to prevent accidental data exposure from `PANGOLIN_NO_AUTH=false`
4. **Environment Variable Consistency**: All handlers must check NO_AUTH the same way (`.is_ok()` vs value comparison)
5. **API Server Compilation**: Fixed 11 compilation errors related to federated catalogs

### Files Created/Modified

**UI Files**:
- `pangolin_ui/src/routes/+layout.svelte` - Main layout
- `pangolin_ui/src/routes/+page.svelte` - Dashboard
- `pangolin_ui/src/routes/login/+page.svelte` - Login page
- `pangolin_ui/src/routes/tenants/+page.svelte` - Tenants management
- `pangolin_ui/src/lib/stores/auth.ts` - Auth state management
- `pangolin_ui/src/lib/stores/theme.ts` - Theme management
- `pangolin_ui/src/lib/api/auth.ts` - Auth API client
- `pangolin_ui/src/lib/components/ui/*` - Reusable components
- `pangolin_ui/tailwind.config.js` - Material Design theme
- `pangolin_ui/src/app.css` - Global styles

**API Files**:
- `pangolin_api/src/user_handlers.rs` - Added `get_app_config()`
- `pangolin_api/src/auth_middleware.rs` - Secure NO_AUTH checks
- `pangolin_api/src/lib.rs` - CORS configuration

**Documentation**:
- `docs/getting-started/env_vars.md` - NO_AUTH documentation
- `docs/authentication.md` - NO_AUTH mode section

---

## 🚧 Phase 2: Resource Management (IN PROGRESS)

**Status**: 🚧 In Progress (50% Complete)  
**Started**: Dec 14, 2025  
**Estimated Completion**: Late Dec 2025 / Early Jan 2026

### Objectives
- Implement CRUD operations for core resources
- Build data tables with sorting/filtering
- Create forms for resource creation
- Handle API errors gracefully

### Features to Implement

#### 2.1 Catalogs Management ✅ COMPLETE
**Priority**: High  
**Pages**: `/catalogs`, `/catalogs/[id]`  
**Status**: ✅ Complete (Dec 14, 2025)

- [x] **List View**
  - Data table with: Name, Warehouse, Storage Location, Created Date
  - Search/filter by name
  - Sort by columns
  - Pagination (if >50 catalogs)
  - "Create Catalog" button (Root/TenantAdmin only)

- [x] **Create Form**
  - ✅ Modal with form
  - ✅ Fields: Name, Warehouse (dropdown), Storage Location
  - ✅ Validation: Required fields, alphanumeric name
  - ✅ Success/error notifications
  - ⏸️ Properties (key-value pairs) - deferred

- [x] **Detail View**
  - Catalog metadata
  - Associated namespaces count
  - Properties display
  - Edit button (Root/TenantAdmin)
  - Delete button (Root only, with confirmation)

- [x] **API Integration**
  - ✅ `GET /api/v1/catalogs` - List catalogs
  - ✅ `POST /api/v1/catalogs` - Create catalog
  - ✅ `GET /api/v1/catalogs/:name` - Get catalog
  - ✅ `DELETE /api/v1/catalogs/:name` - Delete catalog
  - ⏸️ `PUT /api/v1/catalogs/:name` - Update catalog (deferred)

#### 2.2 Warehouses Management ✅ COMPLETE
**Priority**: High  
**Pages**: `/warehouses`, `/warehouses/[id]`  
**Status**: ✅ Complete

- [x] **List View**
  - ✅ Data table with: Name, Type (S3/Azure/GCS), Bucket, Region, Auth
  - ✅ Custom badges for storage type
  - ✅ Custom badges for auth method (Static/IAM)
  - ✅ Search by name
  - ✅ "Create Warehouse" button

- [x] **Create Form**
  - ✅ Multi-step wizard (4 steps: Basic, Storage, Auth, Review)
  - ✅ Storage types: S3, Azure, GCS
  - ✅ S3: Bucket, Region, Custom Endpoint (for MinIO/LocalStack)
  - ✅ Azure: Container, Account Name
  - ✅ GCS: Bucket, Project ID
  - ✅ Authentication methods: Static Credentials OR IAM Role/OAuth
  - ✅ Visual comparison cards showing pros/cons of each auth method
  - ✅ Comprehensive validation
  - ✅ Help text clarifying bucket is for auth, catalogs specify storage
  - ✅ Links to IAM roles documentation

- [ ] **Detail View**
  - Warehouse metadata
  - Storage configuration (mask secrets)
  - Associated catalogs list
  - Edit configuration
  - Delete warehouse (with safety checks)

- [x] **API Integration**
  - ✅ `GET /api/v1/warehouses` - List warehouses
  - ✅ `POST /api/v1/warehouses` - Create warehouse
  - ✅ `GET /api/v1/warehouses/:name` - Get warehouse
  - ⏸️ `DELETE /api/v1/warehouses/:name` - Delete warehouse (deferred)

#### 2.3 Users Management
**Priority**: High  
**Pages**: `/users`, `/users/[id]`

- [ ] **List View**
  - Data table with: Username, Email, Role, Tenant, Last Login, Status
  - Filter by role (Root/TenantAdmin/TenantUser)
  - Filter by tenant (Root only)
  - Search by username/email
  - "Create User" button

- [ ] **Create Form**
  - Fields: Username, Email, Password, Role, Tenant (if Root)
  - Password strength indicator
  - Role descriptions
  - OAuth option (if configured)

- [ ] **Detail View**
  - User profile
  - Assigned roles
  - Permissions summary
  - Activity log (last 10 actions)
  - Edit profile
  - Reset password
  - Deactivate/Delete user

- [ ] **API Integration**
  - `GET /api/v1/users` - List users
  - `POST /api/v1/users` - Create user
  - `GET /api/v1/users/:id` - Get user
  - `PUT /api/v1/users/:id` - Update user
  - `DELETE /api/v1/users/:id` - Delete user

#### 2.4 Tenants Management (Enhancement)
**Priority**: Medium  
**Pages**: `/tenants` (already exists, needs enhancement)

- [ ] **List View Enhancement**
  - Data table with: Name, ID, Users Count, Catalogs Count, Created Date
  - Search by name
  - "Create Tenant" button (Root only)
  - Disabled state in NO_AUTH mode (already done)

- [ ] **Create Form**
  - Fields: Name, Description, Properties
  - Validation: Unique name
  - Note about NO_AUTH restriction

- [ ] **Detail View**
  - Tenant metadata
  - Users list
  - Catalogs list
  - Warehouses list
  - Edit properties
  - Delete tenant (Root only, with safety checks)

### New Components Needed

- [x] **DataTable** - Reusable table with sorting, filtering (pagination deferred)
- [ ] **Form** - Form wrapper with validation (using native forms for now)
- [x] **Select** - Dropdown select component
- [ ] **Checkbox** - Checkbox input (deferred)
- [ ] **Radio** - Radio button input (deferred)
- [x] **Notification/Toast** - Success/error messages
- [x] **ConfirmDialog** - Confirmation modal for destructive actions
- [ ] **Wizard** - Multi-step form component (deferred)
- [x] **LoadingSpinner** - Loading indicator (inline spinner)
- [x] **EmptyState** - Empty list placeholder (built into DataTable)
- [x] **Badge** - Status badges (inline implementation)

### API Client Enhancements

- [x] Create API client modules:
  - [x] `lib/api/catalogs.ts`
  - [x] `lib/api/warehouses.ts`
  - [ ] `lib/api/users.ts` (Phase 2B)
  - [ ] `lib/api/tenants.ts` (Phase 2B)
- [x] Error handling utilities (in base client)
- [x] Loading state management (component-level)
- [ ] Optimistic updates (deferred)

---

## 🚧 Phase 3: Data Operations & Browsing (PLANNED)

**Status**: 📋 Planned  
**Estimated Duration**: 3-4 weeks

### Objectives
- Browse catalog data hierarchy
- View table metadata and schemas
- Explore snapshot history
- Manage namespaces

### Features to Implement

#### 3.1 Namespace Browser
**Priority**: High  
**Pages**: `/catalogs/[catalog]/namespaces`

- [ ] **Hierarchical View**
  - Tree view of nested namespaces
  - Breadcrumb navigation
  - Expand/collapse nodes
  - Search within catalog

- [ ] **List View**
  - Flat list with parent path
  - Filter by depth
  - Sort by name/created date

- [ ] **Create Namespace**
  - Modal form
  - Fields: Name, Parent (optional), Properties
  - Validation: Valid identifier, unique within parent

- [ ] **Namespace Details**
  - Properties display
  - Tables count
  - Sub-namespaces count
  - Edit properties
  - Delete namespace (with safety checks)

#### 3.2 Table/Asset Browser
**Priority**: High  
**Pages**: `/catalogs/[catalog]/namespaces/[namespace]/tables`

- [ ] **Table List**
  - Data table with: Name, Type (Table/View), Schema Version, Last Modified, Size
  - Filter by type
  - Search by name
  - Quick actions: View, Query, Metadata

- [ ] **Table Detail View**
  - **Schema Tab**:
    - Column list with: Name, Type, Nullable, Comment
    - Partition spec
    - Sort order
    - Schema evolution history
  
  - **Snapshots Tab**:
    - Snapshot timeline
    - For each snapshot: ID, Timestamp, Operation, Summary, Manifest Count
    - Time travel to snapshot
    - Compare snapshots
  
  - **Properties Tab**:
    - Table properties (key-value pairs)
    - Table location
    - Format version
    - Current snapshot ID
  
  - **Transactions Tab**:
    - Recent commits
    - User who made change
    - Timestamp
    - Operation type (Append/Overwrite/Delete)
    - Files added/removed

- [ ] **Create Table** (Basic)
  - Schema definition UI
  - Partition configuration
  - Properties

#### 3.3 Branch Management UI
**Priority**: Medium  
**Pages**: `/catalogs/[catalog]/branches`

- [ ] **Branch List**
  - Active branches
  - Branch name, created by, created date, assets tracked
  - Filter by type (ingest/experimental)
  - "Create Branch" button

- [ ] **Create Branch**
  - Fields: Name, Type (ingest/experimental), Source branch, Assets to track
  - Validation: Unique name, valid source

- [ ] **Branch Detail**
  - Assets in branch
  - Commits in branch
  - Merge status
  - Conflicts (if any)
  - Actions: Merge, Delete

- [ ] **Merge Flow**
  - Select target branch
  - Preview changes
  - Conflict detection
  - Conflict resolution UI (if conflicts exist)
  - Complete merge

#### 3.4 Tag Management UI
**Priority**: Low  
**Pages**: `/catalogs/[catalog]/tags`

- [ ] **Tag List**
  - Tag name, asset, snapshot ID, created date
  - Search tags
  - "Create Tag" button

- [ ] **Create Tag**
  - Fields: Name, Asset, Snapshot ID
  - Validation: Unique name per asset

- [ ] **Tag Detail**
  - Tag metadata
  - Associated asset
  - Snapshot details
  - Edit tag
  - Delete tag

### New Components Needed

- [ ] **TreeView** - Hierarchical data display
- [ ] **Breadcrumbs** - Navigation breadcrumbs
- [ ] **Tabs** - Tabbed content
- [ ] **CodeBlock** - Syntax-highlighted code display (for schemas)
- [ ] **Timeline** - Visual timeline for snapshots
- [ ] **DiffViewer** - Compare two snapshots
- [ ] **SchemaEditor** - Visual schema builder

---

## 🚧 Phase 4: Permissions & Advanced Features (PLANNED)

**Status**: 📋 Planned  
**Estimated Duration**: 3-4 weeks

### Objectives
- Implement RBAC UI
- Business metadata and discovery
- Access request workflow
- Audit log viewer

### Features to Implement

#### 4.1 Roles & Permissions
**Priority**: High  
**Pages**: `/roles`, `/permissions`

- [ ] **Roles List**
  - Built-in roles (Root, TenantAdmin, TenantUser)
  - Custom roles
  - Role name, description, users count, permissions count
  - "Create Role" button (Root/TenantAdmin)

- [ ] **Create/Edit Role**
  - Fields: Name, Description
  - Permission builder:
    - Catalog-level permissions
    - Namespace-level permissions
    - Asset-level permissions
    - Tag-based permissions
  - Permission matrix UI
  - Preview affected users

- [ ] **Assign Permissions**
  - User selection
  - Role assignment
  - Direct permission grants
  - Expiration dates (optional)

- [ ] **Permission Viewer**
  - View effective permissions for a user
  - Permission inheritance tree
  - Audit trail

#### 4.2 Business Metadata & Discovery
**Priority**: Medium  
**Pages**: `/search`, `/assets/[id]`

- [ ] **Asset Search**
  - Global search across catalogs
  - Filters: Type, Catalog, Namespace, Tags, Discoverable
  - Search results with: Name, Description, Tags, Access status
  - "Request Access" button for inaccessible assets

- [ ] **Asset Detail (Enhanced)**
  - Business metadata section:
    - Description (rich text editor)
    - Tags/Labels
    - Custom key-value pairs
    - Owner information
  - Discoverability toggle
  - Access requests list (for owners)

- [ ] **Add Business Metadata**
  - Rich text editor for descriptions
  - Tag autocomplete
  - Custom fields builder
  - Save/Cancel

#### 4.3 Access Request Workflow
**Priority**: Medium  
**Pages**: `/access-requests`

- [ ] **Request Access (User)**
  - Modal from search results
  - Fields: Asset, Reason, Requested permissions
  - Submit request

- [ ] **Access Requests List (Admin)**
  - Pending requests
  - Request details: User, Asset, Reason, Requested permissions, Date
  - Actions: Approve, Reject, Request more info
  - Filter by status (Pending/Approved/Rejected)

- [ ] **Request Detail**
  - Full request information
  - User profile
  - Asset details
  - Approval history
  - Approve/Reject with comments

#### 4.4 Audit Log Viewer
**Priority**: Low  
**Pages**: `/audit`

- [ ] **Audit Log List**
  - Event log with: Timestamp, User, Action, Resource, Result
  - Filters: Date range, User, Action type, Resource type
  - Export to CSV
  - Pagination

- [ ] **Audit Detail**
  - Full event details
  - Before/after state (if applicable)
  - Related events
  - User context

### New Components Needed

- [ ] **PermissionMatrix** - Visual permission grid
- [ ] **RichTextEditor** - WYSIWYG editor for descriptions
- [ ] **TagInput** - Tag autocomplete input
- [ ] **DateRangePicker** - Date range selection
- [ ] **ExportButton** - Export data to CSV/JSON
- [ ] **ApprovalFlow** - Approval workflow component

---

## 🚧 Phase 5: Polish & Production Readiness (PLANNED)

**Status**: 📋 Planned  
**Estimated Duration**: 2-3 weeks

### Objectives
- Performance optimization
- Accessibility improvements
- Testing and quality assurance
- Documentation and deployment

### Features to Implement

#### 5.1 Performance Optimization

- [ ] **Code Splitting**
  - Lazy load routes
  - Dynamic imports for heavy components
  - Reduce initial bundle size

- [ ] **Caching Strategy**
  - API response caching
  - Optimistic updates
  - Stale-while-revalidate pattern

- [ ] **Virtual Scrolling**
  - For large lists (>1000 items)
  - Implement in DataTable component

- [ ] **Image Optimization**
  - Generate and use logo (chibi pangolin on iceberg)
  - Optimize all images
  - Lazy load images

#### 5.2 Accessibility (A11y)

- [ ] **Keyboard Navigation**
  - All interactive elements keyboard accessible
  - Focus management
  - Skip links

- [ ] **Screen Reader Support**
  - ARIA labels
  - Semantic HTML
  - Announcements for dynamic content

- [ ] **Color Contrast**
  - WCAG AA compliance
  - Test with contrast checker
  - Adjust theme colors if needed

- [ ] **Focus Indicators**
  - Visible focus states
  - Custom focus rings

#### 5.3 Testing

- [ ] **Unit Tests**
  - Component tests (Vitest)
  - Store tests
  - Utility function tests
  - Target: 80% coverage

- [ ] **Integration Tests**
  - API client tests
  - Form submission flows
  - Authentication flows

- [ ] **E2E Tests**
  - Playwright tests
  - Critical user journeys:
    - Login flow
    - Create catalog
    - Create warehouse
    - Browse tables
    - Request access
  - Target: All critical paths covered

- [ ] **Visual Regression Tests**
  - Screenshot comparison
  - Responsive design tests

#### 5.4 White-Labeling & Customization

- [ ] **Logo Upload**
  - Root user can upload custom logo
  - Replace default pangolin logo
  - Support SVG, PNG

- [ ] **Color Scheme Customization**
  - Root user can customize primary/secondary colors
  - Live preview
  - Save to tenant properties

- [ ] **Branding**
  - Custom app name
  - Custom favicon
  - Custom footer text
**Last Updated**: December 14, 2025 - Phase 2A Catalogs CRUD Complete
#### 5.5 Documentation

- [ ] **User Guide**
  - Getting started
  - Feature walkthroughs
  - Screenshots
  - Video tutorials (optional)

- [ ] **Admin Guide**
  - Installation
  - Configuration
  - User management
  - Troubleshooting

- [ ] **Developer Guide**
  - Architecture overview
  - Component library
  - Contributing guidelines
  - API documentation

#### 5.6 Deployment

- [ ] **Build Optimization**
  - Production build configuration
  - Environment variable management
  - Asset optimization

- [ ] **Docker Support**
  - Dockerfile for UI
  - Docker Compose with API server
  - Health checks

- [ ] **Deployment Guides**
  - Vercel deployment
  - Netlify deployment
  - Self-hosted (Nginx)
  - Kubernetes

### New Components/Features Needed

- [ ] **ErrorBoundary** - Graceful error handling
- [ ] **OfflineIndicator** - Show when offline
- [ ] **UpdateNotification** - Notify when new version available
- [ ] **HelpCenter** - In-app help/documentation
- [ ] **FeedbackWidget** - User feedback collection

---

## 📊 Progress Tracking

### Overall Progress
- **Phase 1**: ✅ 100% Complete
- **Phase 2**: 📋 0% Complete
- **Phase 3**: 📋 0% Complete
- **Phase 4**: 📋 0% Complete
- **Phase 5**: 📋 0% Complete

**Total**: 20% Complete (1/5 phases)

### Estimated Timeline
- **Phase 1**: ✅ Complete (Dec 13-14, 2025)
- **Phase 2**: Jan 2026 (2-3 weeks)
- **Phase 3**: Feb 2026 (3-4 weeks)
- **Phase 4**: Mar 2026 (3-4 weeks)
- **Phase 5**: Apr 2026 (2-3 weeks)

**Estimated Completion**: April 2026

---

## 🎯 Success Criteria

### Phase 1 (Complete)
- [x] Users can log in with JWT
- [x] NO_AUTH mode works for development
- [x] Theme switching functional
- [x] Basic navigation works
- [x] Responsive on mobile/tablet/desktop

### Phase 2
- [ ] Root users can create/manage all resources
- [ ] Tenant admins can manage their tenant resources
- [ ] All CRUD operations work correctly
- [ ] Error handling is user-friendly
- [ ] Forms validate input properly

### Phase 3
- [ ] Users can browse catalog hierarchy
- [ ] Table schemas are viewable
- [ ] Snapshot history is accessible
- [ ] Branches can be created and merged
- [ ] Tags can be managed

### Phase 4
- [ ] RBAC is fully functional
- [ ] Users can search for assets
- [ ] Access requests work end-to-end
- [ ] Audit logs are viewable
- [ ] Business metadata can be added

### Phase 5
- [ ] App loads in <2 seconds
- [ ] WCAG AA compliant
- [ ] 80% test coverage
- [ ] White-labeling works
- [ ] Documentation complete
- [ ] Production-ready deployment

---

## 🔧 Technical Debt & Future Improvements

### Known Issues (from Phase 1)
1. Unit tests failing (auth_test.rs, business_metadata_test.rs) - Axum middleware trait bounds
2. `merge_branch` returns `Result<()>` instead of `Result<Uuid>` - need to return commit ID
3. Minor warnings (unused imports, variables, dead code)

### Future Enhancements
- GraphQL API support
- Real-time updates (WebSockets)
- Advanced query builder
- Data lineage visualization
- Cost analysis dashboard
- Multi-language support (i18n)
- Mobile app (React Native/Flutter)
- CLI tool for power users

---

## 📚 Resources

### Documentation
- [UI Requirements](./ui_requirements.md)
- [Phase 1 Session Summary](./ui_phase1_session_summary.md)
- [RBAC UI Spec](../features/rbac_ui.md)
- [Authentication Docs](../authentication.md)

### External References
- [SvelteKit Docs](https://kit.svelte.dev/)
- [TailwindCSS Docs](https://tailwindcss.com/)
- [Material Design](https://m3.material.io/)
- [Apache Iceberg Spec](https://iceberg.apache.org/spec/)

---

## 🤝 Contributing

This roadmap is a living document. As we progress through each phase, we'll:
1. Update completion status
2. Add lessons learned
3. Adjust timelines based on actual progress
4. Refine future phase plans based on discoveries

**Last Updated**: December 14, 2025
