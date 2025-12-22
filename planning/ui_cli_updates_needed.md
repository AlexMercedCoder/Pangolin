# UI and CLI Updates Required

This document tracks changes needed in the UI and CLI after the API scalability enhancements are complete.

## Background Task Processing

### API Changes
- Maintenance endpoints now return `202 Accepted` with a `task_id` instead of blocking
- New endpoints for task status polling:
  - `GET /api/v1/tasks/:task_id` - Get task status
  - `GET /api/v1/tasks` - List all tasks for a tenant

### UI Updates Needed
- [ ] Update maintenance trigger UI to show "Task submitted" message with task ID
- [ ] Add task status polling component to show progress
- [ ] Add tasks list page to view all background jobs
- [ ] Show task status indicators (pending, running, completed, failed)

### CLI Updates Needed
- [ ] Update maintenance commands to return task ID
- [ ] Add `pangolin-admin task status <task_id>` command
- [ ] Add `pangolin-admin task list` command
- [ ] Add `--wait` flag to maintenance commands to poll until completion

## Error Handling Improvements

### API Changes
- Error responses now include structured JSON with error messages
- HTTP status codes are more granular (400, 401, 403, 404, 409 instead of generic 500)

### UI Updates Needed
- [ ] Update error handling to parse new JSON error format
- [ ] Display more specific error messages based on status codes
- [ ] Add user-friendly error messages for common scenarios

### CLI Updates Needed
- [ ] Parse JSON error responses
- [ ] Display error messages with appropriate formatting
- [ ] Add debug flag to show full error details

## Pagination Support

### API Changes
- `list_assets` and `list_namespaces` now support `?limit=N&offset=M` query parameters
- Response includes pagination metadata: `{ items: [...], total: N, limit: M, offset: K }`

### UI Updates Needed
- [ ] Add pagination controls to asset and namespace list views
- [ ] Implement infinite scroll or page-based navigation
- [ ] Show total count and current page info

### CLI Updates Needed
- [ ] Add `--limit` and `--offset` flags to list commands
- [ ] Add `--all` flag to fetch all results (paginated internally)
- [ ] Show pagination info in output

## Connection Pool Configuration

### API Changes
- New environment variables: `DATABASE_MAX_CONNECTIONS`, `DATABASE_MIN_CONNECTIONS`, `MONGO_MAX_POOL_SIZE`, etc.
- Startup logs now show configured pool sizes

### UI Updates Needed
- No UI changes required (backend-only)

### CLI Updates Needed
- No CLI changes required (backend-only)

## Authorization Middleware

### API Changes
- Some endpoints may return 403 Forbidden earlier in the request lifecycle
- Error messages for authorization failures are more consistent

### UI Updates Needed
- [ ] Ensure 403 responses are handled gracefully
- [ ] Show "Access Denied" message with appropriate context
- [ ] Redirect to login if session expired (401)

### CLI Updates Needed
- [ ] Handle 403 responses with clear error messages
- [ ] Suggest checking permissions or contacting admin

---

**Note**: This document should be reviewed before starting UI/CLI work to ensure all API changes are accounted for.
