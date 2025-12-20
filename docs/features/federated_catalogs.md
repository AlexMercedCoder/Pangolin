# Federated Catalogs

Federated Catalogs allow Pangolin to act as a secure proxy for external Iceberg REST catalogs. This enables centralized governance, unified authentication, and cross-tenant data sharing.

---

## 🛠️ Usage Guides

### 1. Via CLI (`pangolin-admin`)
Manage external connections directly from your terminal.

**Create Federated Catalog:**
```bash
pangolin-admin create-federated-catalog partner_prod \
  --base-url "https://api.partner.io/v1/catalog" \
  --storage-location "s3://partner-bucket/data" \
  --auth-type BearerToken \
  --token "secret_partner_token"
```

**Test Connectivity:**
```bash
pangolin-admin test-federated-catalog partner_prod
```

### 2. Via Management UI
1. Navigate to **Infrastructure -> Catalogs**.
2. Click **Create Catalog** and select **Federated** as the type.
3. Enter the target REST API URL and select your authentication method (Basic, Bearer, or API Key).
4. Click **Test Connection** before saving to ensure Pangolin can reach the target.

---

## 🌐 Cross-Tenant Federation Pattern

A common use case is sharing data between two different Pangolin tenants (e.g., *Engineering* sharing with *Data Science*).

1.  **Source Tenant**: Create a **Service User** and copy its API key.
2.  **Target Tenant**: Create a **Federated Catalog** using the Source's URL and the API key.
3.  **Result**: Users in the Target Tenant can query the Source's data using their own Pangolin credentials.

---

## 🔐 Security & Constraints

- **Single Sign-On**: Users authenticate to their local Pangolin; Pangolin handles the backend handshake with the external catalog.
- **Audit Logging**: All requests passing through the proxy are logged in the *local* tenant's audit trail.
- **Read-Through**: Federated catalogs are strictly for Iceberg REST operations. Pangolin-specific features (like branching or access requests) are not supported on federated assets.

---

## 🚦 Best Practices
- **Timeout Management**: Set a reasonable timeout (default 30s) to prevent slow external catalogs from blocking local resources.
- **Service Users**: Always use a dedicated Service User on the external side rather than a personal account.
- **HTTPS Only**: In production, never use unencrypted HTTP for federated catalog URLs.
