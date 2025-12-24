from .base import BaseAsset

class NimbleAsset(BaseAsset):
    @classmethod
    def register(cls, client, catalog, namespace, name, location, **kwargs):
        return super().register(client, catalog, namespace, name, location, kind="NIMBLE", **kwargs)

class MlModelAsset(BaseAsset):
    @classmethod
    def register(cls, client, catalog, namespace, name, location, **kwargs):
        return super().register(client, catalog, namespace, name, location, kind="ML_MODEL", **kwargs)

class DirectoryAsset(BaseAsset):
    @classmethod
    def register(cls, client, catalog, namespace, name, location, **kwargs):
        return super().register(client, catalog, namespace, name, location, kind="DIRECTORY", **kwargs)

class VideoAsset(BaseAsset):
    @classmethod
    def register(cls, client, catalog, namespace, name, location, **kwargs):
        return super().register(client, catalog, namespace, name, location, kind="VIDEO_FILE", **kwargs)

class ImageAsset(BaseAsset):
    @classmethod
    def register(cls, client, catalog, namespace, name, location, **kwargs):
        return super().register(client, catalog, namespace, name, location, kind="IMAGE_FILE", **kwargs)

class DbConnectionString(BaseAsset):
    @classmethod
    def register(cls, client, catalog, namespace, name, location, **kwargs):
        return super().register(client, catalog, namespace, name, location, kind="DB_CONN_STRING", **kwargs)

class OtherAsset(BaseAsset):
    @classmethod
    def register(cls, client, catalog, namespace, name, location, **kwargs):
        return super().register(client, catalog, namespace, name, location, kind="OTHER", **kwargs)
