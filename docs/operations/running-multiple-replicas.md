# Running more than one replica

Short version: **you can, with PostgreSQL, if you configure session affinity and
accept the caveats below.** Each one is stated with what actually goes wrong,
not just that it is "unsupported".

## What works without any special handling

**The Iceberg commit path.** This is the operation most likely to race, and it
is safe. Table commits use compare-and-swap on the metadata pointer with
requirement enforcement, so two replicas committing to the same table cannot
both win — one gets a commit conflict and retries. That property does not depend
on there being one replica.

**Schema migrations.** PostgreSQL applies its migration chain under a
`pg_advisory_lock`, so replicas starting simultaneously cannot race each other
into a corrupted schema.

**The revocation sweep.** Every replica runs it. `cleanup_expired_tokens` is a
`DELETE ... WHERE expires_at < now`, so concurrent sweeps are idempotent — the
second deletes nothing. Each replica staggers its own start within the interval
so that replicas from one rolling deploy do not sweep in lockstep forever.

> Before 0.8.0 this job **never ran at all**. It was defined, the module was
> declared, and nothing called it, so `revoked_tokens` grew for the life of the
> deployment — and the revocation check reads that table on every authenticated
> request.

## What needs configuration

### OAuth requires session affinity

The OAuth `state` nonce and the one-time code exchange are held **in process**.
If the browser's callback lands on a different replica than the one that started
the flow, the nonce is not found and the login fails with an invalid-state
error.

Configure sticky sessions on your load balancer for the OAuth endpoints, or
disable OAuth and use password or API-key authentication.

```yaml
# Kubernetes Service
spec:
  sessionAffinity: ClientIP
```

This is a known limitation, not a design choice, and it is on the list for the
OIDC work — which needs server-side state for PKCE anyway, and will move both
stores into the database.

### Rate limiting is per replica

The authentication throttle counts in process. With N replicas an attacker
distributing attempts across them gets N times the configured budget. Set
`PANGOLIN_AUTH_RATE_LIMIT` accordingly, and put a limiter in front of the
service if you need a hard number.

### A rotated warehouse credential is served briefly by peers

Warehouses are cached in process for `PANGOLIN_WAREHOUSE_CACHE_TTL_SECS`
(default 5). Invalidation is local, so after you rotate a credential on replica
A, replica B can keep vending the old one for up to the TTL. Five seconds is
deliberately short for exactly this reason. If you need rotation to be
immediate, restart the replicas after rotating.

## What to watch

`pangolin_auth_throttled_total` — a sustained non-zero rate is an attack or a
misconfigured client.

`pangolin_token_revocation_check_errors_total` — revocation currently **fails
open**: if the check errors, the request proceeds. During a database blip every
revoked token is accepted again. This is a known gap (A-13) and this metric is
how you see it happening.

`pangolin_table_commit_cas_retries_total` — rising retries mean replicas are
contending on the same tables. Correct, but it is the signal that you are
scaling writes to one table rather than across tables.

## Recommended configuration

| Setting | Value | Why |
|---|---|---|
| Backend | PostgreSQL | The only backend with a migration chain and full transactional support |
| Replicas | 2–3 to start | Enough for availability; small enough to observe |
| `sessionAffinity` | `ClientIP` | Required if OAuth is enabled |
| `PANGOLIN_JWT_SECRET` | Same across replicas | Otherwise each replica rejects the others' tokens |
| `PANGOLIN_ENCRYPTION_KEY` | Same across replicas | A warehouse sealed by one replica must be readable by all |
| `PANGOLIN_WAREHOUSE_CACHE_TTL_SECS` | 5 (default) | Bounds how long a rotated credential can be served by a peer |

Two of those are worth repeating because getting them wrong fails in confusing
ways: **`PANGOLIN_JWT_SECRET` and `PANGOLIN_ENCRYPTION_KEY` must be identical on
every replica.** A different signing secret makes each replica reject sessions
issued by the others, which looks like random logouts. A different encryption
key makes warehouses written by one replica unreadable by the others, which
looks like intermittent credential-vending failures.

## What has not been tested

Multi-replica operation has **not** been load tested or run for an extended
period. The properties above are established by reading the code and by the test
suite, not by having run three replicas under production traffic for a week.
Treat this page as "the known constraints", not as "this configuration is
proven".
