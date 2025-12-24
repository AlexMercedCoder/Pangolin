from .base import BaseAsset

class DeltaAsset(BaseAsset):
    @classmethod
    def register(cls, client, catalog, namespace, name, location, **kwargs):
        # Register as "DELTA_TABLE" (SCREAMING_SNAKE_CASE expected by API)
        return super().register(client, catalog, namespace, name, location, kind="DELTA_TABLE", **kwargs)

    @classmethod
    def write(cls, client, catalog, namespace, name, data, location, mode="append", **kwargs):
        try:
            from deltalake import write_deltalake
        except ImportError:
            raise ImportError("Please install 'deltalake' (pip install pypangolin[delta]) to use DeltaAsset")
            
        # 1. Write Data
        # write_deltalake(table_or_uri, data, mode='error', ...)
        write_deltalake(location, data, mode=mode, **kwargs)
        
        # 2. Register
        return cls.register(client, catalog, namespace, name, location, **kwargs)
