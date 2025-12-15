# User Access Requests

If you need access to a resource you cannot currently view or modify, you can submit an access request through the CLI.

## Commands

### List Requests
View the status of your submitted access requests.

**Syntax**:
```bash
pangolin-user list-requests
```

### Request Access
Submit a new request for access to a resource.

**Syntax**:
```bash
pangolin-user request-access --resource <resource_identifier> --role <requested_role> --reason <text>
```

**Example**:
```bash
pangolin-user request-access --resource catalog:marketing --role read --reason "Need access for Q4 reporting"
```
The administrator will review your request and grant/deny access.
