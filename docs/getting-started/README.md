# Getting Started with Pangolin

Welcome to Pangolin! This directory contains everything you need to set up, configure, and evaluate the platform.

## 🚀 Onboarding
Get up and running in minutes.
- **[Quick Start Guide](./getting_started.md)**: A step-by-step walkthrough of your first tenant, catalog, and table.
- **[Evaluating Pangolin](./evaluating-pangolin.md)**: Using `NO_AUTH` mode for rapid local testing.

## ⚙️ Configuration
Fine-tune Pangolin for your environment.
- **[Environment Variables](./env_vars.md)**: Comprehensive list of all configuration options.
- **[Configuration Overview](./configuration.md)**: Principles of runtime and storage setup.
- **[Nested Namespaces](./nested_namespaces.md)**: Guide to creating and managing hierarchical namespaces.
- **[Storage Backend Logic](./storage-backend-logic.md)**: How backends determine which credentials to use (vending vs client-side).
- **[Dependencies](./dependencies.md)**: System requirements and library overview.

## 🔌 Client Integration
Connect your favorite tools to the Pangolin REST Catalog.
- **[Client Configuration](./client_configuration.md)**: Setup guides for PyIceberg, PySpark, and Trino.

## 🔐 Authentication Modes
Choose your security model.
- **[Auth Mode](./auth-mode.md)**: Standard operational mode with user authentication.
- **[No Auth Mode](./no-auth-mode.md)**: For local development and testing.

## 🚢 Deployment
Move from local testing to production.
- **[Deployment Guide](./deployment.md)**: Instructions for local, Docker, and production environments.
- **[Docker Deployment](./docker_deployment.md)**: Detailed Docker setup and configuration.

---

### Quick Launch Reference
If you have [Docker](https://www.docker.com/) installed, the fastest way to see Pangolin in action is:

```bash
# Clone and start with Docker Compose
git clone https://github.com/your-org/pangolin.git
cd pangolin
docker-compose up -d
```

Visit the [Quick Start Guide](./getting_started.md) to perform your first data operations!
