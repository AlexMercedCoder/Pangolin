from importlib.metadata import PackageNotFoundError, version as _package_version

from .client import PangolinClient
from .catalog import get_iceberg_catalog

from .assets import (
    BaseAsset,
    DeltaAsset,
    ParquetAsset,
    CsvAsset,
    JsonAsset,
    HudiAsset,
    PaimonAsset,
    LanceAsset,
    VortexAsset,
    NimbleAsset,
    MlModelAsset,
    DirectoryAsset,
    VideoAsset,
    ImageAsset,
    DbConnectionString,
    OtherAsset,
)

# B38: this was hardcoded to "0.1.0" while pyproject.toml published 0.6.0 - the
# "one version everywhere" property 0.6.0 introduced was already broken in the
# SDK one day later. Reading it from the installed distribution means there is
# only one place to change.
try:
    __version__ = _package_version("pypangolin")
except PackageNotFoundError:  # running from a source tree, not installed
    __version__ = "0.0.0+unknown"
__all__ = [
    "PangolinClient", 
    "get_iceberg_catalog",
    "BaseAsset",
    "DeltaAsset",
    "ParquetAsset",
    "CsvAsset",
    "JsonAsset",
    "HudiAsset",
    "PaimonAsset",
    "LanceAsset",
    "VortexAsset",
    "NimbleAsset",
    "MlModelAsset",
    "DirectoryAsset",
    "VideoAsset",
    "ImageAsset",
    "DbConnectionString",
    "OtherAsset",
]
