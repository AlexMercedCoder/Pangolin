# Admin CLI - Token Management

This guide covers token management commands for security and session control.

---

## Revoke Token (Self)

Revoke your own authentication token (logout).

### Syntax
```bash
pangolin-admin revoke-token
```

### Parameters
None

### Example
```bash
# Logout by revoking your own token
pangolin-admin revoke-token
```

### Output
```
✅ Token revoked successfully!
You have been logged out. Please login again to continue.
```

### Notes
- Revokes the currently authenticated token
- Effectively logs you out
- You'll need to login again to use the CLI
- Cannot be undone

---

### Syntax
```bash
pangolin-admin revoke-token-by-id --id <TOKEN_ID>
```

### Parameters
- `--id` - UUID of the token to revoke (required)

### Example
```bash
# Revoke a specific token
pangolin-admin revoke-token-by-id --id "token-uuid"
```

### Output
```
✅ Token token-uuid revoked successfully!
```

---

## List User Tokens (Admin)
List all active tokens for a specific user.

### Syntax
```bash
pangolin-admin list-user-tokens --user-id <USER_ID>
```

### Example
```bash
pangolin-admin list-user-tokens --user-id "user-uuid"
```

---

## Delete Token (Admin)
Forcefully delete a token record.

### Syntax
```bash
pangolin-admin delete-token --token-id <TOKEN_ID>
```

### Example
```bash
pangolin-admin delete-token --token-id "token-uuid"
```

---

## Use Cases

### Self-Logout
```bash
# When done working
pangolin-admin revoke-token
```

### Force User Logout (Security)
```bash
# If a user's credentials are compromised
pangolin-admin revoke-token-by-id --id "compromised-token-uuid"
```

### Session Management
```bash
# End all sessions for a user (requires listing tokens first)
# Note: Token listing endpoint may be needed
pangolin-admin revoke-token-by-id --id "token-1"
pangolin-admin revoke-token-by-id --id "token-2"
```

---

## Security Best Practices

### When to Revoke Tokens

1. **End of Work Session**: Always revoke when done
2. **Security Incident**: Immediately revoke compromised tokens
3. **User Offboarding**: Revoke all tokens when user leaves
4. **Suspicious Activity**: Revoke tokens showing unusual patterns
5. **Regular Rotation**: Periodically revoke and re-issue tokens

### Token Lifecycle

```
Login → Token Issued → Work Session → Revoke Token → Logout
```

### Multi-Device Sessions

Each login creates a new token:
- Desktop: token-1
- Laptop: token-2
- CI/CD: token-3

Revoke individually or all at once.

---

## Workflow Examples

### Normal Logout
```bash
# 1. Finish work
pangolin-admin list-catalogs

# 2. Logout
pangolin-admin revoke-token

# 3. Login again later
pangolin-admin login --username admin --password password
```

### Security Incident Response
```bash
# 1. Login as admin
pangolin-admin login --username admin --password password

# 2. Identify compromised token
# (from security logs or monitoring)

# 3. Revoke the token
pangolin-admin revoke-token-by-id --id "compromised-token-uuid"

# 4. Notify the user
echo "Token revoked. Please contact security."
```

### Scheduled Token Rotation
```bash
#!/bin/bash
# Rotate tokens for all service users weekly

# Get all service user tokens
# (requires token listing endpoint)

# Revoke old tokens
for token_id in "${OLD_TOKENS[@]}"; do
  pangolin-admin revoke-token-by-id --id "$token_id"
done

# Users will need to re-authenticate
```

---

## Error Handling

### Common Errors

**401 Unauthorized**:
```
Error: API Request Failed: Failed to revoke token (401): Not authenticated
```
- Solution: Login first

**403 Forbidden**:
```
Error: API Request Failed: Failed to revoke token (403): Insufficient permissions
```
- Solution: Only admins can revoke other users' tokens

**404 Not Found**:
```
Error: API Request Failed: Failed to revoke token (404): Token not found
```
- Solution: Token may already be revoked or invalid

---

## Comparison with Service User Keys

### Regular Tokens (JWT)
- **Purpose**: User authentication
- **Lifetime**: Session-based
- **Revocation**: Via `revoke-token` commands
- **Use**: Interactive CLI sessions

### Service User API Keys
- **Purpose**: Service-to-service auth
- **Lifetime**: Long-lived (30-90 days)
- **Revocation**: Via `rotate-service-user-key`
- **Use**: Automated systems, CI/CD

---

## Related Commands

- `login` - Authenticate and get a token
- `create-service-user` - Create service user with API key
- `rotate-service-user-key` - Rotate service user API key
- `list-service-users` - List service users

---

## Security Considerations

### Token Storage
- Tokens stored in `~/.pangolin/config`
- Protect this file (chmod 600)
- Never commit to version control

### Token Exposure
If a token is exposed:
1. Immediately revoke it
2. Change your password
3. Review access logs
4. Notify security team

### Audit Trail
- All token revocations are logged
- Review logs regularly
- Monitor for unusual patterns

---

## Best Practices

1. **Always Logout**: Revoke tokens when done
2. **Monitor Sessions**: Track active tokens
3. **Quick Response**: Revoke compromised tokens immediately
4. **Regular Rotation**: Implement token rotation policies
5. **Least Privilege**: Use service users for automation
6. **Audit Regularly**: Review token usage logs
