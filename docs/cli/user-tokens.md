# Token Generation

Generate long-lived JWT tokens for automation, scripts, and external clients.

## User CLI

### Generate Token

Generate a JWT token for your account:

```bash
pangolin-user get-token --description "PyIceberg automation" --expires-in 90
```

**Parameters:**
- `--description` (required): Description of what this token will be used for
- `--expires-in` (optional): Token expiration in days (default: 90)

**Example Output:**
```
✅ Token generated successfully!
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Token: eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Expires: 2025-03-18T12:00:00Z
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

⚠️  Save this token securely. It will not be shown again.
💡 Use this token with PyIceberg:
   catalog = load_catalog('my_catalog', uri='...', token='<TOKEN>')
```

## Using Tokens with PyIceberg

Once you have a token, use it to authenticate with PyIceberg:

```python
from pyiceberg.catalog import load_catalog

catalog = load_catalog(
    "my_catalog",
    **{
        "uri": "http://localhost:8080/v1/my_catalog",
        "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
    }
)

# Now you can use the catalog
table = catalog.load_table("my_namespace.my_table")
df = table.scan().to_arrow()
```

## Security Best Practices

1. **Store tokens securely**: Never commit tokens to version control
2. **Use environment variables**: Store tokens in environment variables or secure vaults
3. **Set appropriate expiration**: Use shorter expiration times for sensitive environments
4. **Rotate tokens regularly**: Generate new tokens periodically
5. **Revoke unused tokens**: Use the token revocation API to revoke tokens you no longer need

## Token Revocation

Tokens can be revoked using the API:

```bash
# Self-revoke (revoke your own token)
curl -X POST http://localhost:8080/api/v1/tokens/revoke/self \
  -H "Authorization: Bearer <TOKEN>"

# Admin revoke (revoke another user's token)
curl -X POST http://localhost:8080/api/v1/tokens/revoke/<JTI> \
  -H "Authorization: Bearer <ADMIN_TOKEN>"
```

## Common Use Cases

### CI/CD Pipelines

Generate a token for your CI/CD pipeline to automate data operations:

```bash
# In your CI/CD setup script
TOKEN=$(pangolin-user get-token --description "GitHub Actions" --expires-in 365)
echo "PANGOLIN_TOKEN=$TOKEN" >> $GITHUB_ENV
```

### Data Science Notebooks

Generate a token for Jupyter notebooks:

```bash
pangolin-user get-token --description "Jupyter notebook" --expires-in 30
```

### Scheduled Jobs

Generate tokens for cron jobs or scheduled tasks:

```bash
pangolin-user get-token --description "Daily ETL job" --expires-in 180
```
