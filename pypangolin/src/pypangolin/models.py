from pydantic import BaseModel, Field, ConfigDict
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
class PermissionScope(BaseModel):
    """A permission scope as the server emits it.

    B_sdk1: the fields used to carry kebab-case aliases (``catalog-id``,
    ``asset-id``, ``tag-name``) which the server never emits, so every scope
    deserialized with all fields ``None`` - a grant on a specific catalog was
    indistinguishable from a tenant-wide one, silently.

    ``PermissionScope`` is ``#[serde(rename_all = "kebab-case", tag = "type")]``,
    and for an enum that renames the *variants*, not the fields of its struct
    variants. So the ``type`` values are kebab-case while the fields are
    snake_case. Verified against the server's actual output rather than inferred
    from the attribute.
    """

    type: str
    catalog_id: Optional[str] = None
    namespace: Optional[str] = None
    asset_id: Optional[str] = None
    tag_name: Optional[str] = None


class PermissionGrant(BaseModel):
    """A scope/actions pair as embedded in a :class:`Role`.

    B_sdk1: ``Role.permissions`` was typed ``List[Permission]``, but the server
    returns ``Vec<PermissionGrant>`` - which has only ``scope`` and ``actions``,
    no ``id``, ``user-id`` or ``granted-by``. Any role that actually had grants
    therefore raised a pydantic ``ValidationError`` on the required fields.
    """

    scope: PermissionScope
    actions: List[str]


class Permission(BaseModel):
    id: Optional[str] = None
    user_id: str = Field(alias="user-id")
    tenant_id: Optional[str] = Field(None, alias="tenant-id")
    actions: List[str]
    scope: PermissionScope
    granted_by: Optional[str] = Field(None, alias="granted-by")
    granted_at: Optional[str] = Field(None, alias="granted-at")


class Role(BaseModel):
    id: Optional[str] = None
    name: str
    description: Optional[str] = None
    tenant_id: Optional[str] = Field(None, alias="tenant-id")
    permissions: List[PermissionGrant] = Field(default_factory=list)

class ServiceUser(BaseModel):
    id: Optional[str] = Field(default=None, alias="service_user_id")
    name: str
    api_key: Optional[str] = None
    description: Optional[str] = None
    role: Optional[str] = None
    active: bool = True
    expires_at: Optional[int] = None
    permissions: List[Permission] = Field(default_factory=list)

    # Improvement #7: `class Config` is deprecated in Pydantic v2 and removed
    # in v3; every import of this module emitted a PydanticDeprecatedSince20
    # warning per model.
    model_config = ConfigDict(populate_by_name=True)

class AccessRequest(BaseModel):
    id: Optional[str] = None
    asset_id: str = Field(alias="asset-id")
    user_id: str = Field(alias="user-id")
    status: str
    requested_at: str = Field(alias="requested-at") # ISO 8601 string
    review_comment: Optional[str] = Field(default=None, alias="review-comment")

    # Improvement #7: `class Config` is deprecated in Pydantic v2 and removed
    # in v3; every import of this module emitted a PydanticDeprecatedSince20
    # warning per model.
    model_config = ConfigDict(populate_by_name=True)

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

    # Improvement #7: `class Config` is deprecated in Pydantic v2 and removed
    # in v3; every import of this module emitted a PydanticDeprecatedSince20
    # warning per model.
    model_config = ConfigDict(populate_by_name=True)

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

    # Improvement #7: `class Config` is deprecated in Pydantic v2 and removed
    # in v3; every import of this module emitted a PydanticDeprecatedSince20
    # warning per model.
    model_config = ConfigDict(populate_by_name=True)

