# Service Users

Service Users are dedicated programmatic identities designed for secure machine-to-machine access. They bypass the standard JWT session flow in favor of long-lived, high-entropy API keys.

---

## 🛠️ Usage Guides

### 1. Via CLI (`pangolin-admin`)
Administrators can provision and rotate keys directly from the terminal.

**Create Service User:**
```bash
pangolin-admin create-service-user \
  --name "etl-pipeline" \
  --role "TenantUser"
```

> [!NOTE]
> Ensure you record the **Service User ID (UUID)** from the output. If the CLI does not display the ID, use the Management UI or the API directly to retrieve it after creation.

**Rotate API Key:**
```bash
pangolin-admin rotate-service-user-key --id <service-user-uuid>
```

### 2. Via Management UI
1. Navigate to **Identity -> Service Users**.
2. Click **Create Service User**, enter a name and expiration date.
3. **Important**: The API key will be displayed in a modal. **Copy it immediately**, as it will never be shown again.
4. **Key Rotation**: To rotate a key, click the **Rotate Key** icon. A new key will be generated and displayed once; the old key is immediately invalidated.
5. To deactivate a user, use the delete action or manage their active status (roadmap feature).

---

## 🔐 Authentication
Service users authenticate by sending the `X-API-Key` header with every request.

```bash
curl -H "X-API-Key: pgl_YOUR_SECRET_KEY" \
     https://your-pangolin-api.com/api/v1/catalogs
```

### Integration: PyIceberg (Standard OAuth2)
PyIceberg supports standard OAuth2, which is the recommended way to use Service Users.

```python
from pyiceberg.catalog import load_catalog

catalog = load_catalog(
    "pangolin",
    **{
        "uri": "http://localhost:8080/v1/default/",
        "credential": "<service_user_uuid>:<api_key>",
        "oauth2-server-uri": "http://localhost:8080/v1/default/v1/oauth/tokens",
        "scope": "catalog",
        "type": "rest",
    }
)
```

### Integration: Custom Clients (X-API-Key)
For simple HTTP clients that don't support OAuth2, you can use the header:
```python
import requests
headers = {"X-API-Key": "pgl_YOUR_SECRET_KEY"}
response = requests.get("https://api.pangolin.io/api/v1/catalogs", headers=headers)
```

---

## 🚦 Best Practices
- **Never Share Keys**: Treat API keys exactly like root passwords.
- **Rotation**: Rotate keys at least every 90 days. If a key is potentially exposed, rotate it immediately.
- **Least Privilege**: Only grant `TenantAdmin` to service users that specifically need to manage other users or infrastructure. Most pipelines only need `TenantUser`.
- **Monitor Usage**: Check the `last_used` timestamp in the UI to identify and prune stale identities.
