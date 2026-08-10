from typing import TYPE_CHECKING

from .client import PangolinClient

if TYPE_CHECKING:  # pragma: no cover - typing only
    from pyiceberg.catalog import Catalog

def get_iceberg_catalog(
    name: str, 
    uri: str = "http://localhost:8080", 
    token: str = None, 
    tenant_id: str = None, 
    credential_vending: bool = True,
    **properties
) -> "Catalog":
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
    
    # B41: imported here rather than at module scope. `__init__.py` imports
    # this module, so a top-level `from pyiceberg.catalog import ...` made the
    # whole Iceberg stack a hard requirement for importing *anything* from
    # pypangolin - including the CLI and its config - and made the test suite
    # uncollectable. Install with `pip install pypangolin[iceberg]` to use this.
    try:
        from pyiceberg.catalog import load_catalog
    except ImportError as e:  # pragma: no cover - depends on the install extras
        raise ImportError(
            "get_iceberg_catalog requires PyIceberg. "
            "Install it with: pip install 'pypangolin[iceberg]'"
        ) from e

    return load_catalog(name, **config)
