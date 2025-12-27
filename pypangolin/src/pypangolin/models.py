from pydantic import BaseModel, Field
from typing import Optional, Dict, List, Any

class Tenant(BaseModel):
    id: str
    name: str
    properties: Dict[str, str] = Field(default_factory=dict)

class Warehouse(BaseModel):
    name: str  # Warehouse names are IDs in some contexts, but let's assume 'name' is the key
    storage_config: Dict[str, str]
    vending_strategy: Optional[Dict[str, Any]] = None # "AwsStatic", {"AwsSts": ...}

class Catalog(BaseModel):
    id: Optional[str] = None
    name: str
    catalog_type: str # "Local", "Federated"
    warehouse_name: Optional[str] = None
    namespace: Optional[str] = None # For federated?
    properties: Dict[str, str] = Field(default_factory=dict)

class Namespace(BaseModel):
    name: List[str] = Field(alias="namespace")
    properties: Dict[str, str] = Field(default_factory=dict)

class Asset(BaseModel):
    id: Optional[str] = None
    name: str
    kind: str # iceberg_table, view, generic_asset types
    location: str
    properties: Dict[str, str] = Field(default_factory=dict)

class User(BaseModel):
    id: str
    username: str
    email: str
    role: str
    tenant_id: Optional[str] = Field(None, alias="tenant-id")

class LoginResponse(BaseModel):
    token: str
    user: Dict[str, Any]

# Phase 4: Git-like Operations Models
class Branch(BaseModel):
    name: str
    head_commit_id: Optional[str] = None
    branch_type: Optional[str] = None
    assets: Optional[List[str]] = None
    catalog_name: Optional[str] = None

class Tag(BaseModel):
    name: str
    commit_id: str
    catalog_name: Optional[str] = None

class Commit(BaseModel):
    id: str
    message: str
    parent_id: Optional[str] = None
    timestamp: Optional[int] = None
    author: Optional[str] = None

class MergeOperation(BaseModel):
    id: str
    source_branch: str
    target_branch: str
    status: str # "in_progress", "completed", "aborted", "conflicted"
    conflicts: Optional[List[Dict[str, Any]]] = None

class Conflict(BaseModel):
    id: str
    asset_name: str
    conflict_type: str # "schema", "data", "metadata"
    details: Dict[str, Any]

# Phase 5: Governance Models
class Permission(BaseModel):
    id: Optional[str] = None
    action: str
    scope_type: str # "global", "catalog", "warehouse", "namespace", "table"
    scope_id: Optional[str] = None

class Role(BaseModel):
    id: Optional[str] = None
    name: str
    permissions: List[Permission] = Field(default_factory=list)

class ServiceUser(BaseModel):
    id: Optional[str] = Field(default=None, alias="service_user_id")
    name: str
    api_key: Optional[str] = None
    expires_at: Optional[int] = None
    permissions: List[Permission] = Field(default_factory=list)

    class Config:
        populate_by_name = True

class AccessRequest(BaseModel):
    id: Optional[str] = None
    asset_id: str = Field(alias="asset-id")
    user_id: str = Field(alias="user-id")
    status: str
    requested_at: str = Field(alias="requested-at") # ISO 8601 string
    review_comment: Optional[str] = Field(default=None, alias="review-comment")

    class Config:
        populate_by_name = True

class BusinessMetadata(BaseModel):
    id: Optional[str] = None
    asset_id: Optional[str] = None
    description: Optional[str] = None
    tags: List[str] = Field(default_factory=list)
    properties: Dict[str, Any] = Field(default_factory=dict)
    discoverable: bool = False
    updated_at: Optional[str] = None # Or int/datetime? API likely returns string or int timestamp
    updated_by: Optional[str] = None

# Phase 6: Admin & System Models
class AuditEvent(BaseModel):
    id: str
    user_id: Optional[str] = None
    action: str
    resource_type: str
    resource_id: Optional[str] = None
    timestamp: str  # ISO 8601
    ip_address: Optional[str] = None
    user_agent: Optional[str] = None
    result: str
    error_message: Optional[str] = None
    metadata: Optional[Dict[str, Any]] = None
    tenant_id: Optional[str] = Field(None, alias="tenant-id")

    class Config:
        populate_by_name = True

class SystemStats(BaseModel):
    catalogs_count: int
    tables_count: int
    namespaces_count: int
    users_count: int
    warehouses_count: int
    branches_count: int
    tenants_count: int
    scope: str

class CatalogSummary(BaseModel):
    name: str
    table_count: int
    namespace_count: int
    branch_count: int
    storage_location: Optional[str] = None

class SearchResult(BaseModel):
    id: str
    name: str
    kind: str
    catalog: str
    namespace: str
    description: Optional[str] = None
    tags: List[str] = Field(default_factory=list)
    has_access: bool
    discoverable: bool

# Phase 7: Federated & Core Enhancement Models
class FederatedCatalogConfig(BaseModel):
    uri: str
    warehouse: Optional[str] = None
    credential: Optional[str] = None
    properties: Dict[str, str] = Field(default_factory=dict)

class FederatedCatalog(BaseModel):
    id: str
    name: str
    config: Optional[FederatedCatalogConfig] = None
    properties: Dict[str, str] = Field(default_factory=dict)

class SyncStats(BaseModel):
    last_sync: Optional[str] = None
    namespaces_synced: int = 0
    tables_synced: int = 0
    errors: int = 0

class View(BaseModel):
    id: Optional[str] = None
    name: str
    sql: str
    schema_: Optional[Dict[str, Any]] = Field(default=None, alias="schema")
    properties: Dict[str, str] = Field(default_factory=dict)

    class Config:
        populate_by_name = True

