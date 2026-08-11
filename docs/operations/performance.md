# Performance

Nobody had measured this. The audit could say "API-key authentication is
O(tenants × service users) bcrypt calls per request" by reading the code, but
not what any of it costs — and a performance claim nobody has measured is a
guess with a number attached.

`scripts/load_test.py` is the harness. It reports **both** client-observed
latency and the server's own histogram, for a reason explained below.

## Measured figures

Developer laptop, Docker, memory backend, 16 concurrent keep-alive connections,
2000 requests per scenario after 200 warm-up:

| Scenario | client p50 | client p95 | client p99 | client req/s | **server mean** |
|---|--:|--:|--:|--:|--:|
| `/health/ready` | 2.4ms | 7.5ms | 13.5ms | 5086 | **0.018ms** |
| `/v1/config` | 2.7ms | 11.6ms | 18.1ms | 3858 | **0.023ms** |
| list catalogs (authenticated) | 4.5ms | 15.0ms | 21.6ms | 2631 | **0.060ms** |

**Read the two columns together.** The client figures include the Python load
generator and the loopback stack; the server-side means come from the server's
own histogram and include only its handling. The gap is roughly 100×, which
means at this scale **the harness is the bottleneck, not the catalog**.

## Why both numbers are reported

The first version of this harness used `urllib.request.urlopen` per request — a
fresh TCP connection each time, dispatched through a thread pool. It reported
~33ms per request and ~500 req/s against a server whose own histogram said 29
**microseconds**.

Those numbers would have been published as "the catalog does 500 req/s". They
were measuring Python. Switching to one keep-alive connection per worker moved
the same server from an apparent 504 req/s to 5086 — a 10× difference that came
entirely from the client.

So the harness now prints the server's own means alongside its own, and says
plainly that a large gap means the generator is the limit. A load test that
cannot tell you which side is slow is not a load test.

## What this does and does not tell you

**Does:** authorization costs something real but small. Adding an authenticated
permission check takes the server-side mean from 0.018ms to 0.060ms — roughly
3×, and still 60 microseconds. Whatever limits throughput in a deployment, it is
not the permission check at this scale.

**Does not:**

- **These are laptop numbers on a memory backend.** PostgreSQL adds a network
  round trip and real I/O per request. Re-measure on your own hardware with your
  own backend before putting anything in an SLA.
- **Writes are not measured.** No commit path, no concurrent-writer contention.
  A load test that creates thousands of tables leaves a database full of them,
  and the interesting write property — commit conflicts under contention — needs
  a harness that coordinates writers on one table. That does not exist yet.
- **API-key authentication is not measured**, which is unfortunate because it is
  the path the audit flagged. It enumerates every tenant and every service user
  per request; the bcrypt cost is now bounded to one round by a key-id prefix,
  but the *database enumeration* is not. With a handful of service users this is
  invisible. It has not been measured with thousands.
- **No sustained soak.** Two thousand requests is a burst, not an hour. Nothing
  here would catch a slow leak or a cache that degrades over time.
- **Single replica.** Multi-replica behaviour is untested; see
  [running-multiple-replicas.md](running-multiple-replicas.md).

## Running it

```bash
python3 scripts/load_test.py --url http://localhost:8080 \
  --user root --password "$PANGOLIN_ROOT_PASSWORD" \
  --concurrency 16 --requests 2000
```

Add `--catalog X --namespace Y --table Z` to include a table read, which is the
path an engine actually exercises and the only one that touches object storage.

Warm-up is not optional. The first authenticated request fills the warehouse
cache and the connection pool; folding it into the sample puts a cold-start
outlier straight into the p99.

## Known cost centres, from reading the code

Unmeasured, listed so they are not forgotten:

1. **API-key authentication enumerates tenants and service users** on every
   request (`auth_middleware.rs`). The bcrypt cost is bounded; the enumeration
   is not.
2. **The revocation check reads `revoked_tokens` on every authenticated
   request.** It is indexed on MongoDB from 0.8.0, and the sweep that keeps the
   table small never actually ran before 0.8.0 either.
3. **Every Iceberg table load reads its metadata file from object storage.**
   That is inherent to the format, and it is why the metadata cache exists.
