# PyIceberg Integration Guides

Pangolin acts as a standard Apache Iceberg REST Catalog, making it compatible with the Python-native **PyIceberg** library.

## 🚀 Getting Started

Choose the guide that matches your environment:

### Authenticated (Production)
For multi-tenant environments where security is enforced via JWT tokens.
- **[Credential Vending](./auth_vended_creds.md)** (Recommended): Pangolin manages storage keys.
- **[Client-Provided Credentials](./auth_client_creds.md)**: You provide storage keys to the client.

### No-Auth (Evaluation)
For rapid local testing and development.
- **[Credential Vending](./no_auth_vended_creds.md)**: Test the vending flow without RBAC.
- **[Client-Provided Credentials](./no_auth_client_creds.md)**: Basic local storage connectivity.

---

## ☁️ Multi-Cloud Support
Detailed configuration for different storage backends.
- **[Azure & GCP Integration](./multi_cloud.md)**: Connecting to ADLS Gen2 and Google Cloud Storage.

---

## 📚 Key Concepts

### X-Iceberg-Access-Delegation
To enable **Credential Vending**, PyIceberg must send the `X-Iceberg-Access-Delegation` header with the value `vended-credentials`. This tells Pangolin that the client expects temporary storage keys for its operations.

### Tenant Context
Pangolin uses either a standard `token` or the custom `X-Pangolin-Tenant` header to determine the tenant context. For PyIceberg, use the `token` property for the best experience.
