from abc import ABC, abstractmethod
from ..client import PangolinClient

class BaseAsset(ABC):
    @classmethod
    @abstractmethod
    def register(cls, client: PangolinClient, catalog: str, namespace: str, name: str, location: str, kind: str, **kwargs):
        """
        Register the asset with the Pangolin Catalog.
        """
        # Ensure namespace is passed as a string (dot-separated) if it's a list, 
        # but the Client.register_asset expects a string for the URL path segment.
        # The Client.namespaces call likely returns lists of strings.
        # Let's assume the user passes a dot-separated string or we join it.
        if isinstance(namespace, list):
            namespace_str = ".".join(namespace)
        else:
            namespace_str = namespace

        # Call the client's register_asset method
        # accessing catalogs -> catalog -> namespaces -> register_asset
        # But wait, our client structure is client.catalogs.namespaces(cat_name).register_asset(...)
        
        return client.catalogs.namespaces(catalog).register_asset(
            namespace=namespace_str,
            name=name,
            kind=kind,
            location=location,
            properties=kwargs.get("properties", {})
        )

    @classmethod
    def write(cls, *args, **kwargs):
        raise NotImplementedError("This asset type does not support writing data yet.")
