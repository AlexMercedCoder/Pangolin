# Additional Backend Work Plan

This document outlines backend API enhancements required to fully support advanced UI features and improve the overall Pangolin experience.

## 1. Token Management
**Goal**: Allow users and admins to view, manage, and revoke access tokens.

### New Endpoints
*   `GET /api/v1/users/me/tokens`
    *   **Description**: List active tokens for the current user.
    *   **Response**: `[{ id, type, created_at, expires_at, name/description }]`
*   `GET /api/v1/users/{user_id}/tokens` (Admin Only)
    *   **Description**: List tokens for a specific user.
*   `DELETE /api/v1/tokens/{token_id}`
    *   **Description**: Revoke a specific token by ID.
    *   **Note**: Currently sending a token works for revocation, but managing them by ID is better.
*   `POST /api/v1/tokens/rotate`
    *   **Description**: Rotate the current token (revoke old, issue new) in one atomic step.

## 2. Enhanced Data Explorer
**Goal**: Provide richer metadata and browsing capabilities without relying solely on Iceberg REST proxy.

### Iceberg Proxy Enhancements
*   **Problem**: Current `listNamespaces` relies on basic Iceberg proxy.
*   **Enhancement**: Ensure `pangolin_api` supports flat and hierarchical namespace listing efficiently.
*   `GET /api/v1/catalogs/{catalog}/namespaces/tree`
    *   **Description**: Return a full tree structure of namespaces for faster UI rendering of deeply nested structures.



## 3. System Configuration & Health
**Goal**: Allow admins to configure system settings via UI.

*   `GET /api/v1/config/settings`
    *   **Description**: Retrieve dynamic system settings (e.g., default warehouse bucket, retention policies).
*   `PUT /api/v1/config/settings`
    *   **Description**: Update system settings.

## 4. Federated Catalogs
**Goal**: Improve management of external catalogs.

*   `POST /api/v1/federated-catalogs/{name}/sync`
    *   **Description**: Trigger an immediate metadata sync for a federated catalog.
*   `GET /api/v1/federated-catalogs/{name}/stats`
    *   **Description**: Get sync history and error counts.

## 5. Branching & Merging
*   `POST /api/v1/branches/{name}/rebase`
    *   **Description**: Rebase a feature branch onto main to resolving conflicts before merge.
## 6. Business Metadata APIs (Registered)
**Goal**: Manage asset metadata and access requests.
*   `GET /api/v1/assets/search`
    *   **Description**: Search for assets with `#tag` support.
*   `POST /api/v1/assets/{id}/request-access`
    *   **Description**: Submit a request to access a restricted asset.
*   `GET /api/v1/access-requests`
    *   **Description**: List access requests (Admin view).
*   `PUT /api/v1/access-requests/{id}`
    *   **Description**: Approve or reject a request.
*   `GET /api/v1/assets/{id}/metadata`
    *   **Description**: Retrieve metadata for an asset.
*   `POST /api/v1/assets/{id}/metadata`
    *   **Description**: Add or update metadata (description, tags, discovery).
