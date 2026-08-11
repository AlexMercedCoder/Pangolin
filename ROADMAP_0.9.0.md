# Roadmap — 0.9.0

0.8.0 was about making the server correct and safe. 0.9.0 is about the two
things 0.8.0 exposed but did not finish: **authorization that is verified rather
than assumed**, and **a release process whose output is what the tag says it
is**.

Every item below was confirmed against the code on `main` at the time of
writing, with the command that confirms it. Nothing here is recalled from
memory — this project's recurring failure is things that look done and are not,
so an item that cannot be demonstrated does not belong on the list.

Ordered by what would hurt most if left undone.

---

## Bucket 1 — Authorization correctness

### A1. `TenantAdmin` bypasses the scope check entirely

`pangolin_api/src/authz.rs:39`

```rust
if session.role == RoleEnum::TenantAdmin {
    // TODO: Check if scope matches session.tenant_id
    return Ok(true);
}
```

Any `TenantAdmin` gets `true` for **any** scope, and `PermissionScope` carries no
tenant: `Catalog { catalog_id }`, `Namespace { catalog_id, namespace }`,
`Asset { catalog_id, namespace, asset_id }`. The function therefore *cannot*
verify the target belongs to the caller's tenant without a lookup it never does.

Whether this is reachable depends on all 41 call sites independently enforcing
tenant scoping first. **That has not been verified, and this is the single most
important thing in 0.9.0.** In a multi-tenant catalog, a cross-tenant read is
the worst outcome the product has.

- Audit all 41 `check_permission(` call sites for prior tenant enforcement.
- Resolve the scope's owning tenant inside `check_permission` and compare it to
  `session.tenant_id`, so correctness does not depend on every caller.
- Add cross-tenant tests to the existing authorization matrix: tenant A's admin
  must be denied on every scope variant belonging to tenant B.

```bash
grep -rn "check_permission(" pangolin/pangolin_api/src/ | grep -v "fn check_permission" | wc -l
```

### A2. `Action::implies` and `Scope::covers` have no adversarial tests

A1's fix rests on `grant.scope.covers(scope)`. The permission matrix tests
confirm allowed things are allowed; they do not systematically confirm that a
narrow grant fails to cover a broad scope. Add the negative direction.

---

## Bucket 2 — Release integrity

The 0.8.0 release published three images, four defects, and a tag that does not
match what shipped. Each item here closes one specific hole that actually opened.

### R1. CI never builds two of the three images

The `docker` job builds `pangolin-api:ci`, starts it, probes shutdown, and fails
if it runs as root. That is a genuine test — and it is the *only* image CI
touches. `Dockerfile.tools` and `pangolin_ui/Dockerfile` are never built in CI at
all, which is exactly why all four 0.8.0 defects were in those two images.

Extend the `docker` job to build and exercise all three:

| Image | Assertion |
|---|---|
| CLI | `pangolin-admin --version` and `--help` exit 0; runs non-root; no `/usr/include/openssl` |
| UI | serves `HTTP 200`; runs non-root; `node_modules` contains no build toolchain |

The UI check is the one that matters most — `npm prune --omit=dev` builds clean
and fails at runtime if a runtime dependency is misfiled as a devDependency.

### R2. Nothing verifies that published artefacts match the tag

Cause of [the 0.8.0 tag/image drift](docs/known-issues/v0.8.0-tag-image-drift.md).
The release must refuse to publish when the working tree differs from the tag
being released, and record the source commit in an OCI label
(`org.opencontainers.image.revision`) so any published image can be traced back.

Resolves the drift as a side effect: 0.9.0's tag and images will agree.

### R3. Images are published from a laptop

`scripts/build_docker_sequential.sh` runs by hand. The 0.8.0 run failed midway
and left the release half-published, and a stray `buildx` process survived a
`pkill` and had to be killed by PID before it pushed an image built from
pre-fix source. Move the push into the tag-triggered workflow that already
builds the binaries.

### R4. `--locked` is not enforced everywhere

Added to `Dockerfile.tools` in 0.8.0. `pangolin/Dockerfile` and the CI build
steps should match, so a published binary is always reproducible from the
committed `Cargo.lock`.

---

## Bucket 3 — Multi-replica correctness

Both of these are documented limitations rather than bugs. They become bugs the
moment someone scales past one replica, which the Helm chart makes easy.

### M1. Rate limiting is per-process

`pangolin_api/src/rate_limit.rs` uses an in-process `moka` cache, so N replicas
give an effective limit of N × the configured value. Documented in the upgrade
guide, but a brute-force limit that silently weakens with scale is the wrong
default. Needs shared backing, or a documented refusal to support it.

### M2. OAuth requires session affinity

The PKCE verifier is held in process — deliberately, since `state` travels
through the browser with the authorization code. Correct for security, but it
means an OAuth login breaks without sticky sessions. Same fix as M1: a shared
store for pending logins.

---

## Bucket 4 — Backend parity

### P1. 59 store-trait methods default to "Operation not supported"

```bash
grep -c "Operation not supported by this store" pangolin/pangolin_store/src/lib.rs
```

Every one is a method some backend may not implement, failing at runtime rather
than compile time. This is how the cloud-credential features shipped without
ever compiling. Audit all 59: which are genuine "this backend cannot do that",
and which are unfinished work wearing the same error message. The parity suite
should assert the intended answer per backend.

### P2. MongoDB branch-create-by-copy is not transactional

PostgreSQL and SQLite got real transactions in 0.8.0; MongoDB falls back to
sequential statements and returns `500` naming the branch. Now that the
replica-set CI job exists, MongoDB transactions are testable.

### P3. `commitTransaction` remains unimplemented

Deliberate, and the only Iceberg REST endpoint still missing. Decide whether
0.9.0 implements it or documents it as permanently out of scope.

---

## Bucket 5 — Debt with a ratchet already in place

Both budgets are enforced in CI and both should move down in 0.9.0.

| Ratchet | Now | Target |
|---|---|---|
| `pangolin/clippy-warning-budget.txt` | 30 (from 314) | 0 |
| `pangolin_ui/svelte-check-budget.txt` | 150 | materially lower |

### D1. Svelte 5 runes: 0 of 90 components

```bash
grep -rl '\$state\|\$derived\|\$props' pangolin_ui/src --include='*.svelte' | wc -l   # 0
find pangolin_ui/src -name '*.svelte' | wc -l                                        # 90
```

The UI runs on Svelte 5 in legacy compatibility mode. It works, and it is a
deprecation clock. 0.8.0 called this migration complete — that was accurate for
*compiling and running* under Svelte 5, but no component uses the new reactivity
model. Migrate in tranches with the `svelte-check` budget ratcheting down.

### D2. Ten remaining `TODO`/`FIXME` markers

```bash
grep -rn "TODO\|FIXME" --include="*.rs" --include="*.svelte" pangolin/ pangolin_ui/src | grep -v /target/ | wc -l
```

A1 is one of them. Triage the rest: fix, convert to a tracked item, or delete.
Notable: `user_handlers.rs:681` (token invalidation), `merge_handlers.rs:281`
(`merge_branch` does not return a commit ID), `oauth_handlers.rs:635` (config
loaded from neither environment nor file).

---

## Bucket 6 — Operational hygiene

### O1. `.env` holds `PANGOLIN_ROOT_PASSWORD` in plaintext

Compose passes it into containers. Flagged during the 0.8.0 audit and not acted
on, because changing it is a decision about how operators are expected to run
the thing, not a bug fix. 0.9.0 should decide: secret file, external secret
manager, or an explicit documented statement that this is the supported way.

### O2. GitHub OAuth cannot be OIDC-validated

GitHub issues no `id_token` and publishes no JWKS, so its logins rest on the
userinfo endpoint while every other provider gets full validation.
`PANGOLIN_OIDC_REQUIRE=true` refuses it. This is a permanent property of the
provider — the work is making the asymmetry obvious in the UI, not just the
docs.

### O3. Publish the GHSA

Requires a person. A fixed version now exists on all three channels, which was
the precondition.

---

## Explicitly not in 0.9.0

- **Re-cutting `v0.8.0`.** Replacing a published tag is the one thing a version
  number exists to prevent. See the known-issue document.
- **1.0.0.** Not until A1 is resolved and the parity audit in P1 is finished.
  A multi-tenant catalog should not call itself 1.0 with an unverified
  cross-tenant authorization path.

---

## How to judge 0.9.0 complete

The standard that caught everything worth catching in 0.8.0: **an item is done
when something executes it, not when something reads it.**

Seven times in 0.8.0 a change compiled, passed all 18 CI jobs, and did not work
— cloud-credential features that had never compiled, a cleanup job that never
started, a release pipeline that never released, two guardrail jobs that never
ran a test, a server that killed itself 25 seconds after boot, a load harness
that understated latency by 1000×, and four defects in two container images that
CI never built. Every one was found by running the artefact.

So: no item above is complete on the strength of a green tick alone. R1 exists
precisely because a green tick was not enough.
