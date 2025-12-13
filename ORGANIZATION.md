# Repository Organization

The Pangolin repository has been organized for better maintainability and clarity.

## Directory Structure

```
pangolin/
├── README.md                    # Project overview
├── architecture.md              # System architecture
├── LICENSE                      # License file
├── example.env                  # Environment variables template
├── test_setup.sh               # Test environment setup script
│
├── docs/                        # Main documentation
│   ├── getting_started.md
│   ├── api_overview.md
│   ├── authentication.md
│   ├── pyiceberg_testing.md
│   ├── security_vending.md
│   └── research/               # Research & planning docs
│       ├── README.md
│       ├── PYICEBERG_INTEGRATION_SUMMARY.md
│       ├── requirements.md
│       ├── AUDIT_AND_TEST_PLAN.md
│       ├── SQLITE_BACKEND_PLAN.md
│       ├── ui_requirements.md
│       └── rest-catalog-open-api.yaml
│
├── tests/                       # Test suites
│   └── pyiceberg/              # PyIceberg integration tests
│       ├── README.md
│       ├── test_client_credentials.py
│       ├── test_warehouse_credentials.py
│       ├── test_read_fix.py
│       ├── test_pyiceberg.py
│       ├── test_pyiceberg_full.py
│       ├── test_no_auth.py
│       ├── test_token_auth.py
│       ├── test_headers.py
│       ├── test_minio_integration.py
│       ├── check_session.py
│       ├── debug_requests.py
│       └── verify_pyiceberg.py
│
├── pangolin/                    # Rust implementation
│   ├── Cargo.toml
│   ├── pangolin_api/           # REST API server
│   ├── pangolin_core/          # Core domain models
│   └── pangolin_store/         # Storage backends
│
└── pangolin_ui/                 # Management UI
    └── (UI files)
```

## What Changed

### Moved to `tests/pyiceberg/`
All Python integration tests:
- `test_*.py` files
- Utility scripts (`check_session.py`, `debug_requests.py`, `verify_pyiceberg.py`)

### Moved to `docs/research/`
Research and planning documentation:
- `PYICEBERG_INTEGRATION_SUMMARY.md`
- `requirements.md`
- `AUDIT_AND_TEST_PLAN.md`
- `SQLITE_BACKEND_PLAN.md`
- `ui_requirements.md`
- `rest-catalog-open-api.yaml`

### Kept in Root
Essential files only:
- `README.md` - Project overview
- `architecture.md` - System architecture
- `LICENSE` - License
- `example.env` - Configuration template
- `test_setup.sh` - Setup script

## Benefits

1. **Cleaner Root Directory** - Only essential files visible
2. **Organized Tests** - All PyIceberg tests in one place with documentation
3. **Separated Concerns** - Research docs separate from user-facing docs
4. **Better Navigation** - Clear structure with READMEs in each directory
5. **Easier Maintenance** - Related files grouped together

## Quick Links

- [Main Documentation](./docs/)
- [PyIceberg Tests](./tests/pyiceberg/)
- [Research Notes](./docs/research/)
- [Getting Started](./docs/getting_started.md)
- [PyIceberg Integration](./docs/pyiceberg_testing.md)
