# Permissions System

Pangolin implements a hierarchical, additive Role-Based Access Control (RBAC) system. It ensures that data is highly secure by default while providing flexibility through cascading grants and Tag-Based Access Control (TBAC).

---

## 🏛️ Permission Hierarchy & Cascading

Permissions in Pangolin are inherited downwards. A grant at a higher level automatically applies to all resources contained within that scope.

| Scope | Includes | Use Case |
| :--- | :--- | :--- |
| **Catalog** | All Namespaces & Assets | Full access for a specific Data Team. |
| **Namespace** | All child Assets (Tables/Views) | Access to a specific "Schema" or project area. |
| **Asset** | Specific Table/View only | Granular access to a single sensitive dataset. |

### 🌊 Cascading Logic
- If you grant `Read` on a **Namespace**, the user can read **every table** existing now OR created in the future within that namespace.
- Grants are **Additive**: If a user is denied `Write` at the Catalog level but granted `Write` on a specific Table, they **can** write to that table.

---

## 🛠️ Usage Guides

### 1. Via CLI (`pangolin-admin`)
The admin tool is the primary way to manage grants.

**Grant Permission:**
```bash
# Grant Read on a specific namespace to a user
pangolin-admin grant-permission <user-id> \
  --action read \
  --scope namespace \
  --catalog production \
  --namespace sales
```

**Revoke Permission:**
```bash
pangolin-admin revoke-permission <permission-id>
```

**List Permissions:**
```bash
pangolin-admin list-permissions --user <user-id>
```

### 2. Via Management UI
1. Log in as a **Tenant Admin**.
2. Navigate to **Identity -> Users** or **Identity -> Roles**.
3. Select a User/Role and click the **Permissions** tab.
4. Click **Add Permission** and select the Scope (Catalog, Namespace, or Asset) and the allowed Actions.

---

## 🏷️ Tag-Based Access Control (TBAC)

Manage access at scale using classification tags rather than individual table grants.

1.  **Tag an Asset**: (e.g., Tag `orders` as `FINANCIAL`).
2.  **Grant Tag Access**: Grant a role `Read` on the `Tag` scope for `FINANCIAL`.
3.  **Result**: Any user with that role can access all assets tagged `FINANCIAL` across all catalogs.

---

## 🧪 Advanced Activity Tracking

Pangolin tracks "Who granted What to Whom" in the **Audit Logs**. Every permission change is a first-class event, allowing security teams to reconstruct the history of access for any dataset.

---

## 🚦 Best Practices
- **Favor Roles over Users**: Create a role like `Data_Analyst` and grant it permissions, then assign users to that role.
- **Start Specific**: Start with `Asset` or `Namespace` level grants before jumping to `Catalog` wide access.
- **Audit Regularly**: Use the `Audit Logs` to verify that permissions are being used as intended.
