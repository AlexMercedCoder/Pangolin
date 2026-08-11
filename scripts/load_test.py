#!/usr/bin/env python3
"""A load harness for the read paths that matter.

C-16's sibling: nobody had ever measured this. The audit could say "API-key
authentication is O(tenants x service users) bcrypt calls per request" from
reading the code, but not what that costs, and a performance claim nobody has
measured is a guess with a number attached.

What this measures, and why these four:

* ``/health/ready``     - the floor. Whatever this costs, everything else costs
                          more; it isolates framework and network overhead from
                          the catalog.
* ``/v1/config``        - the first call every Iceberg client makes.
* list catalogs         - the simplest authenticated read, so the difference
                          from ``/health/ready`` is roughly what authorization
                          costs.
* load table            - the read an engine actually makes in a query, and the
                          one that touches object storage.

Deliberately *not* measured: writes and commits. A load test that creates
thousands of tables leaves a database full of them, and the interesting
write-path property (commit conflicts under contention) needs a different
harness that coordinates concurrent writers on one table.

Usage:
    python3 scripts/load_test.py --url http://localhost:8080 \\
        --user admin --password secret --concurrency 16 --requests 2000
"""

import argparse
import asyncio
import base64
import http.client
import statistics
import sys
import time
import urllib.parse
from dataclasses import dataclass, field


@dataclass
class Result:
    name: str
    latencies_ms: list[float] = field(default_factory=list)
    errors: dict[str, int] = field(default_factory=dict)

    def record_error(self, key: str) -> None:
        self.errors[key] = self.errors.get(key, 0) + 1

    def summary(self, wall_seconds: float) -> str:
        n = len(self.latencies_ms)
        if n == 0:
            return f"{self.name:<24} no successful requests  errors={self.errors}"

        ordered = sorted(self.latencies_ms)

        def pct(p: float) -> float:
            # Nearest-rank. With small samples this is honest about resolution
            # in a way interpolation is not.
            idx = min(len(ordered) - 1, int(p / 100 * len(ordered)))
            return ordered[idx]

        rps = n / wall_seconds if wall_seconds > 0 else 0
        errs = f"  errors={self.errors}" if self.errors else ""
        return (
            f"{self.name:<24} n={n:<6} "
            f"p50={statistics.median(ordered):7.1f}ms  "
            f"p95={pct(95):7.1f}ms  "
            f"p99={pct(99):7.1f}ms  "
            f"max={ordered[-1]:7.1f}ms  "
            f"{rps:7.1f} req/s{errs}"
        )


class Connection:
    """One keep-alive HTTP connection, reused for every request a worker makes.

    The first version of this harness called `urllib.request.urlopen` per
    request. That opens a fresh TCP connection each time and dispatches through
    a thread pool, and it reported ~33ms per request against a server whose own
    histogram said 29 *microseconds* - so the measurement was almost entirely
    the client. Numbers like that are worse than no numbers: they look like a
    slow catalog.

    Reusing the connection is also what a real client does. Iceberg engines hold
    a pooled HTTP client; measuring cold connections per request would not
    describe them either.
    """

    def __init__(self, base_url: str) -> None:
        parsed = urllib.parse.urlparse(base_url)
        self.host = parsed.hostname or "localhost"
        self.port = parsed.port or (443 if parsed.scheme == "https" else 80)
        self.https = parsed.scheme == "https"
        self._conn: http.client.HTTPConnection | None = None

    def _connect(self) -> http.client.HTTPConnection:
        if self._conn is None:
            cls = http.client.HTTPSConnection if self.https else http.client.HTTPConnection
            self._conn = cls(self.host, self.port, timeout=30)
        return self._conn

    def get(self, path: str, headers: dict[str, str]) -> int:
        try:
            conn = self._connect()
            conn.request("GET", path, headers=headers)
            response = conn.getresponse()
            response.read()  # must drain, or the connection cannot be reused
            return response.status
        except Exception:
            # A broken keep-alive connection must not poison every later
            # request from this worker.
            if self._conn is not None:
                self._conn.close()
                self._conn = None
            raise

    def close(self) -> None:
        if self._conn is not None:
            self._conn.close()
            self._conn = None


def server_side_latency(base_url: str) -> dict[str, tuple[int, float]]:
    """Read the server's own histogram, so harness overhead is visible.

    Reporting only client-observed latency hides the difference between "the
    catalog is slow" and "the load generator is slow", which is precisely the
    mistake the first version of this script made.
    """
    conn = Connection(base_url)
    try:
        parsed = urllib.parse.urlparse(base_url)
        c = http.client.HTTPConnection(parsed.hostname or "localhost", parsed.port or 80, timeout=10)
        c.request("GET", "/metrics")
        body = c.getresponse().read().decode("utf-8", "replace")
        c.close()
    except Exception:
        return {}
    finally:
        conn.close()

    sums: dict[str, float] = {}
    counts: dict[str, int] = {}
    for line in body.splitlines():
        if not line.startswith("pangolin_http_request_duration_seconds_"):
            continue
        try:
            labels = line[line.index("{") + 1 : line.index("}")]
            value = float(line.rsplit(" ", 1)[1])
        except (ValueError, IndexError):
            continue
        route = ""
        for part in labels.split(","):
            if part.startswith("route="):
                route = part.split("=", 1)[1].strip('"')
        if not route:
            continue
        if line.startswith("pangolin_http_request_duration_seconds_sum"):
            sums[route] = sums.get(route, 0.0) + value
        elif line.startswith("pangolin_http_request_duration_seconds_count"):
            counts[route] = counts.get(route, 0) + int(value)

    return {r: (counts[r], sums.get(r, 0.0)) for r in counts if counts[r] > 0}


async def drive(
    name: str,
    base_url: str,
    path: str,
    headers: dict[str, str],
    concurrency: int,
    total: int,
) -> tuple[Result, float]:
    """Issue `total` requests across `concurrency` keep-alive connections."""
    result = Result(name)
    loop = asyncio.get_running_loop()

    # One connection per worker, held for the whole run.
    connections = [Connection(base_url) for _ in range(concurrency)]
    per_worker = [total // concurrency] * concurrency
    for i in range(total % concurrency):
        per_worker[i] += 1

    def worker(conn: Connection, count: int) -> tuple[list[float], dict[str, int]]:
        latencies: list[float] = []
        errors: dict[str, int] = {}
        for _ in range(count):
            started = time.perf_counter()
            try:
                status = conn.get(path, headers)
                elapsed_ms = (time.perf_counter() - started) * 1000
                if 200 <= status < 300:
                    latencies.append(elapsed_ms)
                else:
                    errors[f"HTTP {status}"] = errors.get(f"HTTP {status}", 0) + 1
            except Exception as e:  # noqa: BLE001
                key = type(e).__name__
                errors[key] = errors.get(key, 0) + 1
        return latencies, errors

    wall_start = time.perf_counter()
    outcomes = await asyncio.gather(
        *(
            loop.run_in_executor(None, worker, conn, count)
            for conn, count in zip(connections, per_worker)
            if count > 0
        )
    )
    wall = time.perf_counter() - wall_start

    for latencies, errors in outcomes:
        result.latencies_ms.extend(latencies)
        for key, n in errors.items():
            result.errors[key] = result.errors.get(key, 0) + n

    for conn in connections:
        conn.close()

    return result, wall


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--url", default="http://localhost:8080")
    parser.add_argument("--user", help="root basic-auth user")
    parser.add_argument("--password", help="root basic-auth password")
    parser.add_argument("--catalog", default="")
    parser.add_argument("--namespace", default="")
    parser.add_argument("--table", default="")
    parser.add_argument("--concurrency", type=int, default=16)
    parser.add_argument("--requests", type=int, default=500)
    parser.add_argument("--warmup", type=int, default=50)
    args = parser.parse_args()

    base = args.url.rstrip("/")
    headers: dict[str, str] = {"Connection": "keep-alive"}
    if args.user and args.password:
        token = base64.b64encode(f"{args.user}:{args.password}".encode()).decode()
        headers["Authorization"] = f"Basic {token}"
    authed = "Authorization" in headers

    probe = Connection(base)
    try:
        if probe.get("/health/ready", {}) != 200:
            print("the server is not ready", file=sys.stderr)
            return 1
    except Exception as e:  # noqa: BLE001
        print(f"could not reach {base}: {e}", file=sys.stderr)
        return 1
    finally:
        probe.close()

    scenarios: list[tuple[str, str]] = [
        ("health/ready", "/health/ready"),
        ("v1/config", "/v1/config"),
    ]
    if authed:
        scenarios.append(("list catalogs", "/api/v1/catalogs"))
    if args.catalog and args.namespace and args.table:
        scenarios.append(
            (
                "load table",
                f"/v1/{args.catalog}/namespaces/{args.namespace}/tables/{args.table}",
            )
        )

    print(f"target      {base}")
    print(f"concurrency {args.concurrency}  (one keep-alive connection each)")
    print(f"requests    {args.requests} per scenario, after {args.warmup} warm-up")
    print(f"auth        {'basic' if authed else 'none (anonymous scenarios only)'}")
    print()

    before = server_side_latency(base)

    async def run_all() -> None:
        for name, path in scenarios:
            # The first authenticated request fills the warehouse cache and the
            # connection pool; including it would put a cold-start outlier in
            # the p99.
            if args.warmup:
                await drive(name, base, path, headers, args.concurrency, args.warmup)
            result, wall = await drive(
                name, base, path, headers, args.concurrency, args.requests
            )
            print(result.summary(wall))

    asyncio.run(run_all())

    after = server_side_latency(base)
    if before or after:
        print()
        print("Server-side mean, from its own histogram (client figures above")
        print("include the load generator; this does not):")
        for route in sorted(after):
            count_after, sum_after = after[route]
            count_before, sum_before = before.get(route, (0, 0.0))
            delta_count = count_after - count_before
            delta_sum = sum_after - sum_before
            if delta_count > 0:
                mean_ms = (delta_sum / delta_count) * 1000
                print(f"  {route:<40} n={delta_count:<6} mean={mean_ms:8.3f}ms")

    print()
    print("Numbers describe the machine this ran on. Re-measure on yours before")
    print("putting any of them in an SLA. A large gap between the client figures")
    print("and the server-side means says the harness is the bottleneck, not the")
    print("catalog - which is exactly what the first version of this script hid.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
