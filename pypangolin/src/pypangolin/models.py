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
    type: str # "pantry", "glue", "hive", etc.
    warehouse_name: Optional[str] = None
    namespace: Optional[str] = None # For federated?
    properties: Dict[str, str] = Field(default_factory=dict)

class Namespace(BaseModel):
    name: List[str]
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

class LoginResponse(BaseModel):
    token: str
    user: Dict[str, Any]
