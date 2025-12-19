# UI Completion Plan

**Created**: December 19, 2025  
**Status**: 🔄 **READY FOR IMPLEMENTATION**  
**Scope**: Final UI features to achieve feature parity with API/CLI

---

## Context

**API & CLI Status**: ✅ **LOCKED** - All backend endpoints and CLI commands are complete and tested.

**UI Current State**: ~93% complete (41/44 features verified)

**Remaining Work**: 3 primary UI features needed for full feature parity

---

## Outstanding UI Features

### 1. Token Management UI (Profile Page)

**Priority**: HIGH  
**Estimated Effort**: 4-6 hours  
**API Endpoints**: ✅ Available
- `GET /api/v1/users/me/tokens` - List current user's tokens
- `GET /api/v1/users/{user_id}/tokens` - List user tokens (Admin)
- `DELETE /api/v1/tokens/{token_id}` - Delete specific token
- `POST /api/v1/tokens/rotate` - Rotate token (if implemented)

**Implementation**:

#### User Token Management (`/profile/tokens`)
- **Page**: `src/routes/profile/tokens/+page.svelte`
- **Features**:
  - List all active tokens for current user
  - Display: Token ID (truncated), Created At, Expires At, Status
  - "Revoke" button for each token
  - Confirmation dialog before revocation
  - Auto-refresh after revocation
  - Empty state message if no tokens

#### Admin Token Management (`/admin/tokens`)
- **Page**: `src/routes/admin/tokens/+page.svelte`
- **Features**:
  - User selector dropdown
  - List all tokens for selected user
  - Same display/revoke functionality as user view
  - Admin-only access guard

**Components Needed**:
- `src/lib/components/tokens/TokenList.svelte` - Reusable token list
- `src/lib/components/tokens/RevokeTokenDialog.svelte` - Confirmation dialog

**Success Criteria**:
- [ ] User can view their own tokens
- [ ] User can revoke their own tokens
- [ ] Admin can view any user's tokens
- [ ] Admin can revoke any token
- [ ] Proper error handling and feedback
- [ ] Responsive design

---

### 2. Access Request Management UI (Admin)

**Priority**: HIGH  
**Estimated Effort**: 3-4 hours  
**API Endpoints**: ✅ Available
- `GET /api/v1/access-requests` - List all access requests
- `GET /api/v1/access-requests/{id}` - Get specific request
- `PUT /api/v1/access-requests/{id}` - Approve/reject request

**Implementation**:

#### Access Requests Dashboard (`/admin/requests`)
- **Page**: `src/routes/admin/requests/+page.svelte`
- **Features**:
  - List all pending access requests
  - Filter by status (pending, approved, rejected)
  - Display: User, Asset, Reason, Requested At, Status
  - "Approve" and "Reject" buttons
  - Bulk actions (optional)
  - Pagination for large lists

#### Request Detail View (`/admin/requests/[id]`)
- **Page**: `src/routes/admin/requests/[id]/+page.svelte`
- **Features**:
  - Full request details
  - User information
  - Asset details with link
  - Reason/justification
  - Approve/Reject actions
  - Audit trail (who approved/rejected, when)

**Components Needed**:
- `src/lib/components/requests/RequestList.svelte` - Request table
- `src/lib/components/requests/RequestCard.svelte` - Individual request
- `src/lib/components/requests/ApprovalDialog.svelte` - Approve/reject dialog

**Success Criteria**:
- [ ] Admin can view all access requests
- [ ] Admin can filter by status
- [ ] Admin can approve requests
- [ ] Admin can reject requests
- [ ] User receives feedback on request status
- [ ] Proper permission checks

---

### 3. Dashboard Enhancement (PyIceberg Snippet)

**Priority**: MEDIUM  
**Estimated Effort**: 2-3 hours  
**API Endpoints**: N/A (client-side only)

**Implementation**:

#### Getting Started Widget (`/` - Dashboard)
- **Page**: `src/routes/+page.svelte` (enhance existing)
- **Features**:
  - "Getting Started with PyIceberg" card
  - Pre-filled Python code snippet
  - Dynamic values from user context:
    - API URL (from environment or config)
    - Tenant ID (from session)
    - User's first catalog name (if available)
    - Placeholder for token
  - "Copy to Clipboard" button
  - Syntax highlighting
  - Link to full documentation

**Code Snippet Template**:
```python
from pyiceberg.catalog import load_catalog

catalog = load_catalog("pangolin", **{
    "uri": "http://<HOST>:8080/api/v1/catalogs/<CATALOG_NAME>",
    "s3.endpoint": "http://<S3_HOST>:9000",
    "py-iceberg.catalog-impl": "pyiceberg.catalog.rest.RestCatalog",
    "header.X-Pangolin-Tenant": "<TENANT_ID>",
    "token": "<YOUR_ACCESS_TOKEN>"
})

# List namespaces
namespaces = catalog.list_namespaces()
print(namespaces)
```

**Components Needed**:
- `src/lib/components/dashboard/GettingStarted.svelte` - Widget
- `src/lib/components/ui/CodeBlock.svelte` - Syntax highlighted code (reuse if exists)

**Success Criteria**:
- [ ] Snippet displays on dashboard
- [ ] Dynamic values populated correctly
- [ ] Copy button works
- [ ] Syntax highlighting applied
- [ ] Responsive design

---

## Optional Enhancements (Nice-to-Have)

### 4. System Configuration UI (Admin)

**Priority**: LOW  
**Estimated Effort**: 2-3 hours  
**API Endpoints**: ✅ Available
- `GET /api/v1/config/settings`
- `PUT /api/v1/config/settings`

**Implementation**:
- **Page**: `src/routes/admin/settings/+page.svelte`
- **Features**:
  - Form to view/edit system settings
  - Settings: allow_public_signup, default_retention_days, SMTP config
  - Save button with validation
  - Success/error feedback

### 5. Federated Catalog Monitoring UI

**Priority**: LOW  
**Estimated Effort**: 2-3 hours  
**API Endpoints**: ✅ Available
- `GET /api/v1/federated-catalogs/{name}/stats`
- `POST /api/v1/federated-catalogs/{name}/sync`

**Implementation**:
- **Enhancement**: Add to existing `/catalogs` page
- **Features**:
  - "Sync Status" badge for federated catalogs
  - "Sync Now" button
  - Last synced timestamp
  - Sync stats modal (tables synced, errors, etc.)

---

## Implementation Order

### Phase 1: Core Features (Required for 100% Parity)
1. **Token Management UI** (User + Admin) - 4-6 hours
2. **Access Request Management UI** - 3-4 hours
3. **Dashboard PyIceberg Snippet** - 2-3 hours

**Total Estimated Time**: 9-13 hours

### Phase 2: Optional Enhancements
4. **System Configuration UI** - 2-3 hours
5. **Federated Catalog Monitoring** - 2-3 hours

**Total Estimated Time**: 4-6 hours

---

## Technical Guidelines

### Routing
- Follow existing SvelteKit routing patterns
- Use `+page.svelte` for pages
- Use `+page.server.ts` for server-side data loading
- Implement proper loading states

### API Integration
- Use existing `$lib/api.ts` client
- Follow established error handling patterns
- Implement proper loading/error states
- Use SvelteKit's `load` functions for data fetching

### Components
- Follow existing component structure in `$lib/components/`
- Reuse existing UI components (buttons, dialogs, tables)
- Maintain consistent styling with existing pages
- Ensure responsive design

### Authentication & Authorization
- Use existing session management
- Implement proper role checks (Admin vs User)
- Redirect unauthorized users
- Display appropriate error messages

### User Experience
- Provide clear feedback for all actions
- Implement confirmation dialogs for destructive actions
- Show loading states during API calls
- Handle errors gracefully with user-friendly messages
- Maintain consistent UI/UX with existing pages

---

## Testing Strategy

### Manual Testing
- [ ] Test token management as regular user
- [ ] Test token management as admin
- [ ] Test access request approval workflow
- [ ] Test access request rejection workflow
- [ ] Test dashboard snippet with different user contexts
- [ ] Test all features on mobile/tablet viewports

### Integration Testing
- [ ] Verify API integration for all new endpoints
- [ ] Test error scenarios (network failures, auth errors)
- [ ] Verify permission checks work correctly
- [ ] Test with different user roles

---

## Success Metrics

**Completion Criteria**:
- ✅ All 3 core UI features implemented
- ✅ UI Testing Matrix shows 100% (44/44 features)
- ✅ No console errors
- ✅ Responsive design verified
- ✅ All manual tests pass
- ✅ Documentation updated

**Definition of Done**:
- Code reviewed and merged
- Manual testing complete
- UI Testing Matrix updated
- User documentation updated (if needed)
- No known bugs or regressions

---

## Dependencies

**Required**:
- ✅ API endpoints (all available)
- ✅ Authentication system (working)
- ✅ Existing UI components (available)
- ✅ SvelteKit routing (configured)

**None** - All dependencies are satisfied. Ready to implement!

---

## Notes

- **API/CLI are locked**: Do not modify backend or CLI during UI implementation
- **Consistency**: Follow existing UI patterns and component structure
- **Incremental**: Implement and test one feature at a time
- **User-Centric**: Focus on clear, intuitive user experience
- **Mobile-First**: Ensure all features work on mobile devices

---

## Next Steps

1. Review and approve this plan
2. Begin with Token Management UI (highest priority)
3. Test thoroughly before moving to next feature
4. Update UI Testing Matrix as features are completed
5. Create walkthrough documentation when all features are done
