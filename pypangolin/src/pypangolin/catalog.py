from pyiceberg.catalog import Catalog, load_catalog
from .client import PangolinClient

def get_iceberg_catalog(name: str, uri: str = None, token: str = None, **properties) -> Catalog:
    """
    Helper to initialize a PyIceberg catalog with Pangolin defaults.
    """
    config = {
        "uri": uri,
        "token": token,
        **properties
    }
    # TODO: Fetch token if not provided but username/password are?
    
    return load_catalog(name, **config)
