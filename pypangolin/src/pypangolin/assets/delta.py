from .base import BaseAsset

class DeltaAsset(BaseAsset):
    @classmethod
    def register(cls, client, catalog, namespace, name, location, **kwargs):
        # TODO: Call client.catalogs.get(catalog).namespaces.get(namespace).register_asset(...)
        pass

    @classmethod
    def write(cls, client, catalog, namespace, name, data, location, mode="append", **kwargs):
        try:
            from deltalake import write_deltalake
        except ImportError:
            raise ImportError("Please install 'deltalake' (pip install pypangolin[delta]) to use DeltaAsset")
            
        # 1. Write Data
        write_deltalake(location, data, mode=mode, **kwargs)
        
        # 2. Register
        cls.register(client, catalog, namespace, name, location)
