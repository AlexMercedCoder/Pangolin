# Encrypting warehouse credentials at rest

A warehouse holds the credentials Pangolin uses to reach your object storage:
AWS secret access keys, Azure account keys, GCP service account JSON. Before
0.8.0 these were stored in the catalog database as plaintext JSON, so anything
that could read one row of the `warehouses` table held every tenant's cloud
credentials — a backup, a read replica, a snapshot, an analyst with `SELECT`.

From 0.8.0 they are encrypted with AES-256-GCM when you configure a key.

## Turning it on

```bash
openssl rand -base64 32
```

Set the result as `PANGOLIN_ENCRYPTION_KEY` and restart. The server logs a
warning at startup while it is unset, because a security control that silently
does nothing is worse than one that is visibly absent.

```yaml
environment:
  - PANGOLIN_ENCRYPTION_KEY=${PANGOLIN_ENCRYPTION_KEY:?generate with: openssl rand -base64 32}
```

The key belongs in a secret manager, not in `.env` and not in the image. It is
the only thing standing between a database dump and your customers' cloud
accounts.

## What it protects, and what it does not

| Threat | Protected |
|---|:--:|
| Stolen database backup or snapshot | ✅ |
| Read replica, or an operator with `SELECT` | ✅ |
| SQL injection elsewhere in the application | ✅ |
| Full compromise of a running server | ❌ |
| Compromise of the secret manager holding the key | ❌ |

The key lives in the server's environment, so an attacker who owns the running
process can read it and decrypt everything. That is the normal limit of envelope
encryption without an HSM or a cloud KMS, and it is stated here rather than
implied away. What this buys you is that the *database* is no longer sufficient
on its own.

## Existing warehouses

Turning encryption on does not rewrite anything. Reads tolerate plaintext, so
every existing warehouse keeps working — an upgrade must not be an outage. But
those rows stay in plaintext until something writes them again.

To seal them, update each warehouse once. Any update does it, including one that
changes nothing meaningful:

```bash
pangolin-admin warehouse update <name> --use-sts false
```

Or through the API, re-submitting the storage config:

```bash
curl -X PUT "$PANGOLIN_URL/api/v1/warehouses/$NAME" \
  -H "Authorization: Bearer $TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{"storage_config": { ... }}'
```

To find what still needs it, look for rows whose credential fields do not begin
with `enc:v1:`:

```sql
-- PostgreSQL
SELECT tenant_id, name
FROM warehouses
WHERE storage_config->>'secret_access_key' NOT LIKE 'enc:v1:%'
   OR storage_config->>'account_key'       NOT LIKE 'enc:v1:%'
   OR storage_config->>'client_secret'     NOT LIKE 'enc:v1:%';
```

## Losing the key

There is no recovery. The credentials are gone and the warehouses must be
recreated with fresh credentials from your cloud provider. Back the key up
wherever you back up your other break-glass secrets, and treat it with the same
care as `PANGOLIN_JWT_SECRET`.

If you start the server with the *wrong* key, reads fail loudly rather than
returning rubbish — GCM authenticates the ciphertext — with an error naming
`PANGOLIN_ENCRYPTION_KEY` as the likely cause.

## Rotating the key

There is no online rotation yet. To change keys: decrypt with the old key by
running with it set, re-submit every warehouse's storage config to bring the
values into memory, stop, set the new key, and re-submit again. For a small
number of warehouses this is minutes of work; for a large estate, wait for
proper rotation support rather than scripting this.

## What is encrypted

Only the credential-bearing entries of `storage_config`. The bucket, container,
region, endpoint and account name stay readable: they are not secrets, the
object-store factory compares and concatenates them, and encrypting them would
break storage access for no gain.

Covered keys, in both the dotted and undotted spellings that appear in real
configurations:

`secret_access_key` · `access_key_id` · `session_token` · `account_key` ·
`client_secret` · `service_account_json` · `external_id`

If you use a storage backend whose credential is not on that list, it is **not**
being encrypted. Say so in an issue and it will be added; the list is an
allowlist precisely so that non-secrets stay usable, and the cost of that choice
is that a new secret has to be added deliberately.

## Backends

PostgreSQL, SQLite and MongoDB all seal on write and open on read. The memory
backend does not: it keeps everything in a `DashMap` and loses it on restart, so
there is no "at rest" to protect.

`pangolin_store/tests/warehouse_encryption_tests.rs` reads the raw stored bytes
through its own database connection — not through the store — and asserts the
plaintext is absent. A backend added later that forgets to call `secrets::seal`
fails that test rather than quietly storing credentials in the clear.
