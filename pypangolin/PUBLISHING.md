# Publishing PyPangolin to PyPI

This guide outlines the steps to build and publish the `pypangolin` client library to the Python Package Index (PyPI).

## Prerequisites

1.  **PyPI Account**: Ensure you have an account on [pypi.org](https://pypi.org/).
2.  **API Token**: Generate an API token in your account settings (scoped to the project if it exists, or entire account for first publish).
3.  **Build Tools**: Install `hatch` (recommended) or `build` + `twine`.

```bash
pip install hatch
# OR
pip install build twine
```

## Versioning

Before publishing, ensure `pyproject.toml` has the correct version number.

```toml
[project]
name = "pypangolin"
version = "0.1.0"  # <--- Update this
```

## Option 1: Using Hatch (Recommended)

Since this project uses `hatchling` as the build backend, `hatch` provides the smoothest workflow.

### 1. Build
Creates the `.whl` and `.tar.gz` files in `dist/`.

```bash
hatch build
```

### 2. Publish
Uploads the contents of `dist/` to PyPI.

```bash
hatch publish
```

*First time setup*: It will prompt for your username (`__token__`) and your API token.

## Option 2: Using Twine (Standard)

If you prefer standard Python tooling:

### 1. Build

```bash
python -m build
```

### 2. Verify (Optional)
Check the distribution for errors.

```bash
twine check dist/*
```

### 3. Upload

```bash
twine upload dist/*
```

## CI/CD Publishing (GitHub Actions)

To automate publishing on release, add a workflow `.github/workflows/publish-pypi.yml`:

```yaml
name: Publish to PyPI
on:
  release:
    types: [published]

jobs:
  pypi-publish:
    runs-on: ubuntu-latest
    permissions:
      id-token: write
    steps:
      - uses: actions/checkout@v4
      - name: Set up Python
        uses: actions/setup-python@v4
        with:
          python-version: "3.x"
      - name: Install dependencies
        run: pip install hatch
      - name: Build and Publish
        run: hatch publish
        env:
          HATCH_INDEX_USER: __token__
          HATCH_INDEX_AUTH: ${{ secrets.PYPI_API_TOKEN }}
```

## Checklist

- [ ] Version bump in `pyproject.toml`.
- [ ] Update `README.md` / `CHANGELOG.md`.
- [ ] Run tests (`hatch run test` or `pytest`).
- [ ] Verified local install (`pip install .`).
