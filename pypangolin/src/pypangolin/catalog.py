from pyiceberg.catalog import Catalog, load_catalog
from .client import PangolinClient

def get_iceberg_catalog(
    name: str, 
    uri: str = "http://localhost:8080", 
    token: str = None, 
    tenant_id: str = None, 
    credential_vending: bool = True,
    **properties
) -> Catalog:
    """
    Initialize a PyIceberg catalog with Pangolin defaults.
    
    Args:
        name: Name of the catalog in Pangolin
        uri: Base URI of the Pangolin API (e.g., http://localhost:8080)
        token: JWT Token for authentication
        tenant_id: Optional Tenant ID for header injection
        credential_vending: Whether to request vended credentials (default: True)
        **properties: Additional PyIceberg catalog properties
        
    Returns:
        Configured PyIceberg Catalog instance
    """
    # Construct REST URI
    # Server routes are /v1/:prefix/... where prefix is catalog name.
    # PyIceberg appends /v1/config, so we point to /v1/{name}
    rest_uri = f"{uri.rstrip('/')}/v1/{name}"
    
    # Base config
    config = {
        "uri": rest_uri,
        "type": "rest", # Force REST type
    }
    
    # Auth
    if token:
        config["token"] = token
        
    # Tenant Context
    if tenant_id:
        config["header.X-Pangolin-Tenant"] = tenant_id
        
    # Credential Vending
    if credential_vending:
        config["header.X-Iceberg-Access-Delegation"] = "vended-credentials"
        
    # Merge with user properties (user props override defaults if needed)
    config.update(properties)
    
    return load_catalog(name, **config)
