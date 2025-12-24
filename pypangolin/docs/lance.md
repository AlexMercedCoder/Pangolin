# Lance Assets

Pangolin supports Lance (and LanceDB) datasets.

## Dependencies

Install helper library:
```bash
pip install "pypangolin[lance]"
# or
pip install lancedb
```

## Usage

Use `LanceAsset` to write data via `lancedb` and register the dataset.

```python
import pandas as pd
from pypangolin.assets import LanceAsset

df = pd.DataFrame({"vector": [[1.1, 2.2], [3.3, 4.4]], "label": ["a", "b"]})

# Writes a new Lance dataset and registers it
LanceAsset.write(
    client=client,
    catalog="my_catalog",
    namespace="my_ns",
    name="my_vectors",
    data=df,
    location="/tmp/my_vectors.lance", # Local or S3 URI
    properties={"vector_dim": "1536", "model": "openai-ada-002"}
)
```

## Recommended Properties

| Property | Description | Example |
|---|---|---|
| `dataset_version` | Lance format version | `2` |
| `row_count` | Number of vectors/rows | `10000` |
| `vector_dim` | Dimension of vectors | `1536` |
| `index_type` | Indexing algorithm used | `IVF_PQ` |
| `model_name` | Model used for embeddings | `text-embedding-ada-002` |

