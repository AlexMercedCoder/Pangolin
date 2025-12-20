# Security & Identity

This guide covers how to manage identities and access control within the Pangolin UI.

## 👤 User Management

Manage the lifecycle of human users in your tenant.
- **Invite Users**: Create accounts with a default role.
- **Profile Details**: View a user's role, home tenant, and recent activity.
- **Revocation**: Delete or deactivate users (Root/Admin only).

## 🎭 Role-Based Access Control (RBAC)

Pangolin uses a dynamic RBAC model to grant precise permissions.

### Roles
The UI provides a dedicated **Roles** dashboard where you can:
- **Define Roles**: Create a "Data Scientist" or "Auditor" role.
- **Assign Roles to Users**: One user can hold multiple roles simultaneously.

### Permissions Interface
When creating or editing a role, the UI provides a granular selector for:
- **Action**: `Read`, `Write`, `Create`, `Delete`, `List`, `Manage`.
- **Scope Type**: `Global`, `Catalog`, `Namespace`, `Asset`, or `Tag`.
- **Target**: The specific resource (e.g., the `finance` namespace).

---

## 🔑 Token Management

Manage your session and programmatic access via JWT tokens.

### Personal Tokens
From the **Profile** menu, users can:
- **List Sessions**: See all active web and API sessions.
- **Revoke Tokens**: Manually sign out other devices or rotate your current session.

### Admin Token Control
Tenant Admins can view and revoke tokens for **any user** within their tenant to mitigate security incidents or during user offboarding.

---

## 🛡️ Identity Providers

Pangolin's UI seamlessly handles multiple authentication flows:
- **Native**: Username and Bcrypt-hashed password.
- **OAuth**: One-click login with Google, GitHub, or Microsoft (configured in [System Settings](./administration.md)).
