# Contributing to Pangolin

Thanks for looking. Pangolin has 134 pages of user documentation and, until
recently, nothing at all for contributors — this file closes that gap.

## From clone to green

You need Rust 1.94 or newer and Docker (only for the database-backed tests).

```bash
git clone https://github.com/AlexMercedCoder/pangolin
cd pangolin/pangolin

cargo build --workspace
cargo test --workspace          # ~330 tests, no external services required
```

A clean checkout must be green with nothing but a Rust toolchain. If it is not,
that is a bug — please report it. Tests that genuinely need a database skip with
a printed note rather than failing.

To run those too:

```bash
cd ..                           # repository root
docker compose -f docker-compose.db-test.yml up -d postgres mongo

cd pangolin
export PANGOLIN_TEST_POSTGRES_URL=postgresql://testuser:testpass@localhost:5432/testdb
export PANGOLIN_TEST_MONGO_URL=mongodb://testuser:testpass@localhost:27017
cargo test --workspace
```

Each backend has its own variable. They used to share `DATABASE_URL`, which
meant PostgreSQL and MongoDB could never be satisfied at the same time.

MongoDB parity is incomplete; four MongoDB tests are known-failing. See
**Known limitations** in the README.

## Before you push

CI runs all of these, so running them locally is faster than finding out later:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets
cargo test --workspace
```

Formatting is enforced. `clippy` is not yet blocking, but the warning count is
ratcheted: `pangolin/clippy-warning-budget.txt` records the current total and CI
fails if it grows. If you remove warnings, lower the number in the same commit.

`git config blame.ignoreRevsFile .git-blame-ignore-revs` keeps the one
workspace-wide formatting commit out of your `git blame` output.

## Repository layout

| Path | What it is |
|---|---|
| `pangolin/` | The Rust workspace — six crates |
| `pangolin/pangolin_core` | Domain model: assets, permissions, audit, Iceberg metadata |
| `pangolin/pangolin_store` | The `CatalogStore` trait and its four backends |
| `pangolin/pangolin_api` | HTTP surface: Iceberg REST, Pangolin extensions, auth |
| `pangolin/pangolin_cli_*` | Admin and user CLIs |
| `pypangolin/` | Python SDK |
| `pangolin_ui/` | SvelteKit UI |
| `deployment_assets/helm/` | Helm chart |
| `docs/` | User and operator documentation |

Two files are worth reading before a substantial change:

* `pangolin/pangolin_store/src/lib.rs` — the `CatalogStore` trait every backend
  implements.
* `AUDIT_EXECUTION_PLAN.md` — a candid assessment of what is weak and why, with
  a phased plan. If you are looking for something worth doing, start there.

## Conventions

**Commits.** Conventional-commit prefixes (`feat:`, `fix:`, `docs:`, `chore:`,
`ci:`, `test:`, `refactor:`). Write the body for someone who will read it in a
year with no memory of the discussion: say what changed and *why the previous
behaviour was wrong*, not just what the new behaviour is.

**Comments.** Explain reasoning, not mechanics. A comment that restates the code
is noise; a comment explaining why a check has to happen before an `await`, or
why a foreign key is deliberately absent, earns its place.

**Errors.** Never discard a `Result` with `let _ = ...` in a path where the
failure matters. Audit writes in particular must be logged when they fail. Avoid
`unwrap()` and `expect()` outside tests and startup.

**Tests.** Every bug fix gets a test that fails before it and passes after. Name
the test after the property it protects, not the function it calls
(`assert_ref_snapshot_id_rejects_a_stale_writer`, not `test_commit_2`).

Backend tests must skip, not fail, when their database is absent.

## Adding a `CatalogStore` method

The trait carries default implementations that return
`Err("Operation not supported by this store")`. That means a missing method
compiles cleanly and surfaces only as an opaque runtime 500, so:

1. Implement it in **all four** backends, or document the gap in the backend
   parity matrix in `docs/operations/backend-parity.md`.
2. Add it to `CachedCatalogStore` in `pangolin_api/src/cached_store.rs`. That
   wrapper is hand-written delegation; a method you forget there falls through
   to the trait default and silently disables a working backend feature the
   moment caching is enabled.
3. Never default to a *successful* empty result. Returning `Ok(vec![])` from an
   unimplemented search told users their data was missing.

## Database schema changes

There is exactly one source of truth per backend — see
`pangolin/pangolin_store/migrations/README.md`. Do not add a second.

* **PostgreSQL**: add `migrations/<UTC timestamp>_<description>.sql`. `sqlx`
  records checksums, so never edit a migration that has shipped.
* **SQLite**: edit `sql/sqlite_schema.sql` using `IF NOT EXISTS`, and bump
  `SQLITE_SCHEMA_VERSION`.
* **MongoDB**: no schema; document any new index.

## Reporting security issues

Do not open a public issue. See [SECURITY.md](SECURITY.md).

## Code of conduct

Be straightforward and kind. Critique code, not people. Assume the person you
are replying to is acting in good faith and knows things you do not. Maintainers
will act on harassment, personal attacks, or sustained bad faith.

## Licence

Contributions are accepted under the MIT licence, the same as the project.
