# Pangolin Documentation

Welcome to the comprehensive documentation for Pangolin, a multi-tenant Apache Iceberg REST Catalog with advanced features including federated catalogs, credential vending, fine-grained permissions, and Git-like branching.

## 📚 Documentation Index

### 🚀 Getting Started

New to Pangolin? Start here!

- **[Getting Started Guide](./getting-started/getting_started.md)** - Quick start guide
- **[Evaluating Pangolin](./getting-started/evaluating-pangolin.md)** - Feature comparison and evaluation
- **[Installation](./getting-started/installation.md)** - Installation instructions
- **[Configuration](./getting-started/configuration.md)** - Configuration guide
- **[Deployment](./getting-started/deployment.md)** - Deployment options
- **[Docker Setup](./getting-started/docker.md)** - Running with Docker
- **[First Catalog](./getting-started/first_catalog.md)** - Create your first catalog
- **[First Table](./getting-started/first_table.md)** - Create your first table

### 🏗️ Architecture

Understand how Pangolin works.

- **[Architecture Overview](./architecture/README.md)** - System architecture and components
- **[Data Models](./architecture/models.md)** - Complete catalog of all data models
- **[CatalogStore Trait](./architecture/catalog-store-trait.md)** - Core storage abstraction
- **[Signer Trait](./architecture/signer-trait.md)** - Credential vending interface

### 💾 Backend Storage

Choose and configure your metadata storage backend.

- **[Storage Overview](./backend_storage/README.md)** - Backend comparison and selection guide
- **[In-Memory Store](./backend_storage/memory.md)** - Development and testing
- **[SQLite](./backend_storage/sqlite.md)** - Embedded and edge deployments
- **[PostgreSQL](./backend_storage/postgresql.md)** - Production deployments
- **[MongoDB](./backend_storage/mongodb.md)** - Cloud-native and scalable deployments
- **[Comparison](./backend_storage/comparison.md)** - Detailed feature comparison

### 🗄️ Warehouse Storage

Configure cloud storage for your data.

- **[Warehouse Overview](./warehouse/README.md)** - Storage configuration guide
- **[AWS S3](./warehouse/s3.md)** - Amazon S3 configuration
- **[Azure Blob Storage](./warehouse/azure.md)** - Azure configuration
- **[Google Cloud Storage](./warehouse/gcs.md)** - GCP configuration

### 🔌 API Reference

REST API documentation and integration.

- **[API Overview](./api/README.md)** - API introduction
- **[OpenAPI Specification](./api/openapi.json)** - Complete OpenAPI 3.0 spec (JSON)
- **[OpenAPI YAML](./api/openapi.yaml)** - Complete OpenAPI 3.0 spec (YAML)
- **[Swagger UI](http://localhost:8080/swagger-ui)** - Interactive API documentation (when running)
- **[Authentication](./authentication.md)** - JWT and OAuth authentication

### 🔐 Security & Access Control

Authentication, authorization, and permissions.

- **[Authentication Guide](./authentication.md)** - JWT tokens and OAuth
- **[Permissions](./permissions.md)** - Fine-grained RBAC
- **[Service Users](./service_users.md)** - API keys for services
- **[Federated Catalogs](./federated_catalogs.md)** - Remote catalog integration

### ⚙️ Features

Advanced capabilities and features.

- **[Branch Management](./features/branch_management.md)** - Git-like branching
- **[Tag Management](./features/tag_management.md)** - Named snapshots
- **[Merge Operations](./features/merge_operations.md)** - Branch merging and conflict resolution
- **[Time Travel](./features/time_travel.md)** - Query historical data
- **[Credential Vending](./features/credential_vending.md)** - Automatic credential management
- **[Business Metadata](./features/business_metadata.md)** - Asset discovery and cataloging
- **[Audit Logging](./features/audit_logging.md)** - Security and compliance
- **[Multi-Tenancy](./features/multi_tenancy.md)** - Tenant isolation
- **[Federated Catalogs](./features/federated_catalogs.md)** - Remote catalog proxying

### 🐍 PyIceberg Integration

Using Pangolin with PyIceberg.

- **[PyIceberg Overview](./pyiceberg/README.md)** - Integration guide
- **[Setup Guide](./pyiceberg/setup.md)** - Configuration
- **[Examples](./pyiceberg/examples.md)** - Code examples
- **[Testing](./pyiceberg/testing.md)** - Test results and validation

### 💻 CLI Tools

Command-line interface documentation.

- **[CLI Overview](./cli/README.md)** - CLI tools introduction
- **[Admin CLI](./cli/admin_cli.md)** - Administrative commands
- **[Catalog CLI](./cli/catalog_cli.md)** - Catalog operations
- **[User Management](./cli/user_management.md)** - User commands
- **[Permission Management](./cli/permission_management.md)** - Permission commands

### 🖥️ Management UI

Web-based management interface.

- **[UI Overview](./ui/README.md)** - Management UI guide
- **[Getting Started](./ui/getting_started.md)** - UI setup
- **[Features](./ui/features.md)** - UI capabilities
- **[Development](./ui/development.md)** - UI development guide

### 🛠️ Utilities

Developer tools and utilities.

- **[Utilities Overview](./utilities/README.md)** - Available utilities
- **[Regenerating OpenAPI](./utilities/regenerating-openapi.md)** - OpenAPI spec generation

### 📖 Concepts

Core concepts and terminology.

- **[Concepts Overview](./concepts/README.md)** - Key concepts explained

### 🔧 Setup & Configuration

Detailed setup and configuration guides.

- **[Setup Overview](./setup/README.md)** - Setup guides

---

## 🎯 Quick Navigation by Task

### I want to...

#### Get Started
- **Install Pangolin** → [Installation Guide](./getting-started/installation.md)
- **Run with Docker** → [Docker Setup](./getting-started/docker.md)
- **Create my first catalog** → [First Catalog](./getting-started/first_catalog.md)
- **Evaluate Pangolin** → [Evaluation Guide](./getting-started/evaluating-pangolin.md)

#### Configure Storage
- **Choose a backend** → [Backend Comparison](./backend_storage/comparison.md)
- **Set up PostgreSQL** → [PostgreSQL Guide](./backend_storage/postgresql.md)
- **Configure S3** → [S3 Configuration](./warehouse/s3.md)
- **Use Azure Blob** → [Azure Configuration](./warehouse/azure.md)

#### Integrate with Tools
- **Use PyIceberg** → [PyIceberg Setup](./pyiceberg/setup.md)
- **Use Spark** → [Spark Integration](./features/spark_integration.md)
- **Use Trino** → [Trino Integration](./features/trino_integration.md)

#### Manage Access
- **Set up authentication** → [Authentication Guide](./authentication.md)
- **Configure permissions** → [Permissions Guide](./permissions.md)
- **Create service users** → [Service Users](./service_users.md)
- **Enable OAuth** → [OAuth Setup](./features/oauth.md)

#### Use Advanced Features
- **Create branches** → [Branch Management](./features/branch_management.md)
- **Merge branches** → [Merge Operations](./features/merge_operations.md)
- **Time travel queries** → [Time Travel](./features/time_travel.md)
- **Vend credentials** → [Credential Vending](./features/credential_vending.md)

#### Develop & Extend
- **Understand architecture** → [Architecture Overview](./architecture/README.md)
- **View data models** → [Data Models](./architecture/models.md)
- **Regenerate OpenAPI** → [OpenAPI Regeneration](./utilities/regenerating-openapi.md)
- **Run tests** → [Testing Guide](./getting-started/testing.md)

---

## 📊 Documentation Status

| Section | Status | Last Updated |
|---------|--------|--------------|
| Getting Started | ✅ Complete | 2025-12 |
| Architecture | ✅ Complete | 2025-12 |
| Backend Storage | ✅ Complete | 2025-12 |
| Warehouse Storage | ✅ Complete | 2025-12 |
| API Reference | ✅ Complete | 2025-12 |
| Security & Access | ✅ Complete | 2025-12 |
| Features | ✅ Complete | 2025-12 |
| PyIceberg | ✅ Complete | 2025-12 |
| CLI Tools | ✅ Complete | 2025-12 |
| Management UI | ✅ Complete | 2025-12 |
| Utilities | ✅ Complete | 2025-12 |

---

## 🔗 External Resources

- **[Apache Iceberg](https://iceberg.apache.org/)** - Apache Iceberg project
- **[PyIceberg](https://py.iceberg.apache.org/)** - Python client for Iceberg
- **[OpenAPI Specification](https://swagger.io/specification/)** - OpenAPI 3.0 spec
- **[Rust Documentation](https://doc.rust-lang.org/)** - Rust language docs

---

## 🤝 Contributing

Found an issue or want to improve the documentation?

1. Check existing documentation for accuracy
2. Submit improvements via pull request
3. Follow the documentation style guide
4. Update the index if adding new pages

---

## 📝 Documentation Conventions

- **Bold** - Important concepts or actions
- `Code` - Commands, file paths, and code references
- ✅ - Completed or recommended
- ⚠️ - Warning or consideration
- ❌ - Not recommended or not supported

---

## 🆘 Getting Help

- **Issues** - Report bugs or request features on GitHub
- **Discussions** - Ask questions in GitHub Discussions
- **Documentation** - Search this documentation
- **Examples** - Check the examples in each guide

---

**Last Updated**: December 2025  
**Version**: 1.0.0  
**Maintainers**: Pangolin Team
