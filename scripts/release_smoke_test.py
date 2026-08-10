#!/usr/bin/env python3
"""Verify a released Pangolin image over HTTP.

This is the gate `docker-compose.release.yml` runs against a published image
before that image is considered good.

It replaces `integration_test.py` in that role. That script cannot work there
and never could: it builds and launches its own server with `cargo run`, from
paths like `./pangolin/target/debug/pangolin_api`, inside a `python:3.11-slim`
container that has only `./scripts` mounted and no Rust toolchain. B10 fixed the
compose file pointing at a script that did not exist, and replaced it with one
that cannot run in that container - so the release verification step has never
verified a release.

Everything here talks to an already-running server over HTTP, which is the only
thing that container can do, and is also the more useful test: it exercises the
shipped artifact rather than a fresh build of the source.

Configuration:
    PANGOLIN_API_URL   base URL of the server (default http://localhost:8080)
    PANGOLIN_NO_AUTH   "true" to also run the authenticated-surface round trip
    SMOKE_TIMEOUT      seconds to wait for readiness (default 120)
"""

import json
import os
import sys
import time
import uuid

import requests

API_URL = os.environ.get("PANGOLIN_API_URL", "http://localhost:8080").rstrip("/")
NO_AUTH = os.environ.get("PANGOLIN_NO_AUTH", "false").lower() == "true"
TIMEOUT = int(os.environ.get("SMOKE_TIMEOUT", "120"))

failures: list[str] = []


def check(name: str, ok: bool, detail: str = "") -> bool:
    """Record a result. Returns ok, so callers can branch on it."""
    if ok:
        print(f"  PASS  {name}")
    else:
        print(f"  FAIL  {name}{': ' + detail if detail else ''}")
        failures.append(name)
    return ok


def wait_for_ready() -> bool:
    """Block until the server reports ready, or the timeout expires.

    `/health/ready` calls through to the store, so a 200 here means the server
    is up *and* its database is reachable - which is the property worth gating
    a release on. `/health` alone returns 200 regardless (A-21).
    """
    deadline = time.time() + TIMEOUT
    last = "no response"
    while time.time() < deadline:
        try:
            r = requests.get(f"{API_URL}/health/ready", timeout=5)
            if r.status_code == 200:
                print(f"  ready after {int(TIMEOUT - (deadline - time.time()))}s: {r.text.strip()}")
                return True
            last = f"HTTP {r.status_code}: {r.text.strip()[:120]}"
        except requests.RequestException as e:
            last = str(e)
        time.sleep(2)
    print(f"  never became ready within {TIMEOUT}s; last response: {last}")
    return False


def main() -> int:
    print(f"Release smoke test against {API_URL}")
    print(f"  no-auth mode: {NO_AUTH}")
    print()

    print("Readiness")
    if not check("server becomes ready (and its store is reachable)", wait_for_ready()):
        # Nothing below can mean anything if the server never came up.
        print("\nFAILED: server never became ready")
        return 1

    print("\nLiveness and observability")
    try:
        r = requests.get(f"{API_URL}/health/live", timeout=10)
        check("/health/live returns 200", r.status_code == 200, f"HTTP {r.status_code}")
    except requests.RequestException as e:
        check("/health/live returns 200", False, str(e))

    try:
        r = requests.get(f"{API_URL}/metrics", timeout=10)
        # A Prometheus endpoint that exists but exports nothing is not useful,
        # so assert on content as well as status.
        check("/metrics returns 200", r.status_code == 200, f"HTTP {r.status_code}")
        check(
            "/metrics exports at least one series",
            r.status_code == 200 and any(
                line and not line.startswith("#") for line in r.text.splitlines()
            ),
            "no non-comment lines in the exposition",
        )
    except requests.RequestException as e:
        check("/metrics returns 200", False, str(e))

    print("\nIceberg REST surface")
    try:
        # The config endpoint is what an Iceberg client calls first. If this is
        # not wired, no engine can use the catalog at all.
        r = requests.get(f"{API_URL}/v1/config", timeout=10)
        ok = check(
            "/v1/config responds", r.status_code in (200, 401), f"HTTP {r.status_code}"
        )
        if ok and r.status_code == 200:
            body = r.json()
            check(
                "/v1/config returns the spec's defaults/overrides shape",
                isinstance(body, dict) and "defaults" in body and "overrides" in body,
                f"got keys {sorted(body)[:6]}",
            )
    except (requests.RequestException, json.JSONDecodeError) as e:
        check("/v1/config responds", False, str(e))

    if NO_AUTH:
        print("\nCatalog round trip")
        name = f"smoke-{uuid.uuid4().hex[:8]}"
        try:
            r = requests.post(
                f"{API_URL}/api/v1/catalogs",
                json={"name": name, "catalog_type": "Local", "properties": {}},
                timeout=15,
            )
            created = check(
                "create a catalog", r.status_code in (200, 201), f"HTTP {r.status_code}: {r.text[:160]}"
            )

            if created:
                r = requests.get(f"{API_URL}/api/v1/catalogs", timeout=15)
                names = []
                if r.status_code == 200:
                    payload = r.json()
                    rows = payload if isinstance(payload, list) else payload.get("data", [])
                    names = [c.get("name") for c in rows]
                check(
                    "the new catalog appears in the listing",
                    name in names,
                    f"HTTP {r.status_code}, saw {names[:5]}",
                )

                # Clean up so a re-run against the same server is not affected
                # by the previous one.
                r = requests.delete(f"{API_URL}/api/v1/catalogs/{name}", timeout=15)
                check(
                    "delete the catalog", r.status_code in (200, 204), f"HTTP {r.status_code}"
                )
        except (requests.RequestException, json.JSONDecodeError) as e:
            check("catalog round trip", False, str(e))
    else:
        print("\nCatalog round trip")
        print("  SKIP  set PANGOLIN_NO_AUTH=true to exercise the authenticated surface")

    print()
    if failures:
        print(f"FAILED: {len(failures)} check(s): {', '.join(failures)}")
        return 1
    print("All checks passed.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
