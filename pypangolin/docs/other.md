# Other Assets

Pangolin supports a wide variety of generic assets beyond standard table formats.

## Available Types

- `NimbleAsset`
- `MlModelAsset`
- `DirectoryAsset`
- `VideoAsset`
- `ImageAsset`
- `DbConnectionString`
- `OtherAsset` (Catch-all)

## Usage

All these assets follow the same registration pattern.

```python
from pypangolin.assets import MlModelAsset, VideoAsset, DbConnectionString

# Register an ML Model
MlModelAsset.register(
    client=client,
    catalog="ml_catalog",
    namespace="models",
    name="churn_predictor",
    location="s3://bucket/models/churn/v1",
    properties={
        "framework": "pytorch",
        "version": "1.0.0",
        "accuracy": "0.95"
    }
)

# Register a Video
VideoAsset.register(
    client=client,
    catalog="media_catalog",
    namespace="raw",
    name="demo_recording",
    location="s3://bucket/videos/demo.mp4",
    properties={
        "duration_seconds": "120",
        "resolution": "1080p",
        "codec": "h264"
    }
)

# Register a Database Connection
DbConnectionString.register(
    client=client,
    catalog="infra_catalog",
    namespace="connections",
    name="prod_postgres",
    location="postgresql://user:pass@host:5432/db", # Be careful with secrets!
    properties={
        "environment": "production",
        "db_type": "postgres"
    }
)
```

## Recommended Properties

### ML Models
| Property | Description | Example |
|---|---|---|
| `framework` | ML Framework | `pytorch`, `tensorflow`, `sklearn` |
| `version` | Model version | `1.0.0` |
| `accuracy` | Performance metric | `0.98` |
| `input_shape` | Input tensor shape | `(1, 3, 224, 224)` |

### Video Files
| Property | Description | Example |
|---|---|---|
| `duration_seconds` | Length of video | `3600` |
| `resolution` | Video resolution | `1920x1080` |
| `codec` | Encoding codec | `h264`, `av1` |
| `frame_rate` | Frames per second | `30`, `60` |

### Images
| Property | Description | Example |
|---|---|---|
| `width` | Image width (px) | `1024` |
| `height` | Image height (px) | `768` |
| `format` | File format | `png`, `jpg`, `webp` |

### Database Connections
| Property | Description | Example |
|---|---|---|
| `db_type` | Database engine | `postgres`, `mysql`, `snowflake` |
| `environment` | Deployment env | `prod`, `staging` |
| `host` | Hostname (if not in URI) | `db.example.com` |
