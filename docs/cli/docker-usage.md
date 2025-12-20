# Using Pangolin CLI via Docker

This guide explains how to use the Pangolin CLI tools (`pangolin-admin` and `pangolin-user`) via the Docker container instead of installing binaries locally.

## Docker Image

The Pangolin CLI tools are available in the `alexmerced/pangolin-cli` Docker image:

```bash
docker pull alexmerced/pangolin-cli:0.1.0
```

## Available Commands

The Docker image includes both CLI tools:
- `pangolin-admin` - Administrative operations (tenants, warehouses, catalogs, users, permissions)
- `pangolin-user` - User operations (discovery, access requests, tokens, branches)

## Basic Usage

### Running Admin Commands

```bash
docker run --rm alexmerced/pangolin-cli:0.1.0 pangolin-admin --help
```

### Running User Commands

```bash
docker run --rm alexmerced/pangolin-cli:0.1.0 pangolin-user --help
```

## Configuration

### Environment Variables

Pass configuration via environment variables:

```bash
docker run --rm \
  -e PANGOLIN_API_URL=http://your-api:8080 \
  -e PANGOLIN_TOKEN=your_jwt_token \
  alexmerced/pangolin-cli:0.1.0 \
  pangolin-admin list-tenants
```

### Configuration File

Mount a configuration file into the container:

```bash
docker run --rm \
  -v $(pwd)/config.json:/root/.pangolin/config.json \
  alexmerced/pangolin-cli:0.1.0 \
  pangolin-admin list-tenants
```

## Common Workflows

### Interactive Mode

For interactive sessions, use the `-it` flags:

```bash
docker run -it \
  -e PANGOLIN_API_URL=http://localhost:8080 \
  alexmerced/pangolin-cli:0.1.0 \
  pangolin-admin
```

### Scripting

Create a shell alias for easier usage:

```bash
# Add to your .bashrc or .zshrc
alias pangolin-admin='docker run --rm -e PANGOLIN_API_URL=http://localhost:8080 alexmerced/pangolin-cli:0.1.0 pangolin-admin'
alias pangolin-user='docker run --rm -e PANGOLIN_API_URL=http://localhost:8080 alexmerced/pangolin-cli:0.1.0 pangolin-user'

# Then use like normal commands
pangolin-admin list-tenants
pangolin-user search "my dataset"
```

### Docker Compose Integration

Include CLI tools in your `docker-compose.yml`:

```yaml
services:
  cli:
    image: alexmerced/pangolin-cli:0.1.0
    environment:
      - PANGOLIN_API_URL=http://pangolin-api:8080
      - PANGOLIN_TOKEN=${PANGOLIN_TOKEN}
    command: ["echo", "CLI tools ready. Use: docker compose run cli pangolin-admin --help"]
    depends_on:
      - pangolin-api
```

Run commands:

```bash
docker compose run --rm cli pangolin-admin list-tenants
docker compose run --rm cli pangolin-user search "customers"
```

## Network Considerations

### Connecting to Local API

When the API is running on your host machine:

**Linux:**
```bash
docker run --rm \
  --network host \
  -e PANGOLIN_API_URL=http://localhost:8080 \
  alexmerced/pangolin-cli:0.1.0 \
  pangolin-admin list-tenants
```

**Mac/Windows:**
```bash
docker run --rm \
  -e PANGOLIN_API_URL=http://host.docker.internal:8080 \
  alexmerced/pangolin-cli:0.1.0 \
  pangolin-admin list-tenants
```

### Connecting to Dockerized API

When using Docker Compose or a custom network:

```bash
docker run --rm \
  --network your_network_name \
  -e PANGOLIN_API_URL=http://pangolin-api:8080 \
  alexmerced/pangolin-cli:0.1.0 \
  pangolin-admin list-tenants
```

## Examples

### Create a Tenant

```bash
docker run --rm \
  -e PANGOLIN_API_URL=http://localhost:8080 \
  -e PANGOLIN_TOKEN=your_root_token \
  alexmerced/pangolin-cli:0.1.0 \
  pangolin-admin create-tenant --name "acme"
```

### List Catalogs

```bash
docker run --rm \
  -e PANGOLIN_API_URL=http://localhost:8080 \
  -e PANGOLIN_TOKEN=your_token \
  alexmerced/pangolin-cli:0.1.0 \
  pangolin-admin list-catalogs
```

### Search for Datasets

```bash
docker run --rm \
  -e PANGOLIN_API_URL=http://localhost:8080 \
  -e PANGOLIN_TOKEN=your_token \
  alexmerced/pangolin-cli:0.1.0 \
  pangolin-user search "sales"
```

### Generate a Token

```bash
docker run --rm \
  -e PANGOLIN_API_URL=http://localhost:8080 \
  -e PANGOLIN_TOKEN=your_admin_token \
  alexmerced/pangolin-cli:0.1.0 \
  pangolin-admin generate-token --user-id user_123 --expiry 7d
```

## Advantages of Docker CLI

✅ **No Installation**: No need to install Rust or compile binaries
✅ **Consistent Environment**: Same version across all platforms
✅ **Easy Updates**: `docker pull` to get the latest version
✅ **Isolation**: Doesn't affect your system
✅ **CI/CD Friendly**: Easy to integrate into pipelines

## Disadvantages

❌ **Slower Startup**: Docker overhead on each command
❌ **Longer Commands**: More verbose than native binaries
❌ **Network Complexity**: Requires understanding Docker networking

## When to Use Docker vs Binaries

**Use Docker when:**
- You don't want to install binaries
- Running in CI/CD pipelines
- Need consistent environments across teams
- Already using Docker for other tools

**Use Binaries when:**
- Running many commands frequently
- Want faster execution
- Prefer simpler command syntax
- Working locally without Docker

## Troubleshooting

### Cannot Connect to API

Check network connectivity:
```bash
docker run --rm --network host alpine ping -c 3 localhost
```

### Permission Denied

Ensure your user is in the docker group:
```bash
sudo usermod -aG docker $USER
# Log out and back in
```

### Token Issues

Verify your token is valid:
```bash
curl -H "Authorization: Bearer $PANGOLIN_TOKEN" http://localhost:8080/api/v1/tenants
```

## Related Documentation

- [Admin CLI Reference](admin.md)
- [User CLI Reference](user.md)
- [Configuration Guide](configuration.md)
- [Binary Installation](../deployment_assets/bin/README.md)
