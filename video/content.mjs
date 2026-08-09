// The five scripts.
//
// Every claim here is traceable to the 0.6.0 repository: README.md's maturity
// table, CHANGELOG.md's 0.6.0 entry and "Still outstanding" list, SECURITY.md's
// advisory and known-gaps section, and docs/operations/backend-parity.md.
// Nothing is rounded up. If a thing is beta, the video says beta.
//
// Markup allowed inside copy: <u> accent (display faces), <k> accent bold and
// <q> muted (mono rows). Nothing else — the renderer escapes the rest.

export const videos = [
  {
    slug: "what-is-pangolin",
    title: "What is Pangolin?",
    series: "Pangolin // 01 — Overview",
    ghost: "catalog",
    scenes: [
      {
        type: "open",
        wordmark: "PAN<u>GOLIN</u>",
        tagline: "An open Iceberg REST catalog, written in Rust.",
        specs: [
          ["Version", "0.6.0"],
          ["Status", "Alpha"],
          ["License", "MIT"],
        ],
      },
      {
        type: "statement",
        kicker: "The job",
        big: "A catalog is the <u>index</u> of your lakehouse.",
        sub: "It tells query engines which tables exist, where their metadata lives, and who is allowed to touch them.",
      },
      {
        type: "list",
        kicker: "What it is",
        head: "One binary that speaks Iceberg REST",
        rows: [
          "Implements the <k>core</k> of the Apache Iceberg REST spec <q>— not all of it</q>",
          "A single Rust binary. No JVM, no runtime to install",
          "Multi-tenant: tenant scope is a required parameter throughout",
          "MIT licensed, and a passion project rather than a product",
        ],
      },
      {
        type: "list",
        kicker: "What it adds",
        head: "Beyond a plain REST catalog",
        rows: [
          "<k>Git-style branching</k>, tags and 3-way merge over catalog metadata",
          "<k>Federated catalogs</k> — proxy an external Iceberg REST endpoint",
          "<k>Credential vending</k> — AWS STS, Azure SAS, GCP downscoped tokens",
          "Business metadata, asset search, RBAC and audit logging",
        ],
      },
      {
        type: "statement",
        kicker: "And what it is not",
        big: "Alpha software. <u>Not production-proven.</u>",
        sub: "Pangolin is pre-1.0 and under active hardening. The README ships a maturity table that names every gap.",
      },
      {
        type: "close",
        big: "Read the honest version <u>before</u> you deploy it.",
        repo: "github.com/AlexMercedCoder/Pangolin",
      },
    ],
  },

  {
    slug: "architecture-and-features",
    title: "Architecture & Features",
    series: "Pangolin // 02 — Architecture",
    ghost: "shape",
    scenes: [
      {
        type: "open",
        wordmark: "ARCHI<u>TECTURE</u>",
        tagline: "One Rust workspace, a pluggable store, and warehouses on three clouds.",
        specs: [
          ["Language", "Rust"],
          ["Unsafe", "Forbidden"],
          ["Tests", "334 passing"],
        ],
      },
      {
        type: "list",
        kicker: "The workspace",
        head: "Six crates, one binary you actually run",
        rows: [
          "<k>pangolin_core</k> <q>— domain models and the CatalogStore trait</q>",
          "<k>pangolin_store</k> <q>— the backend implementations and migrations</q>",
          "<k>pangolin_api</k> <q>— the REST server you deploy</q>",
          "<k>pangolin-admin</k> and <k>pangolin-user</k> <q>— the two CLIs</q>",
        ],
      },
      {
        type: "table",
        kicker: "Metadata backends",
        head: "Where the catalog itself lives",
        rows: [
          ["PostgreSQL", "Recommended"],
          ["SQLite — single writer", "Good"],
          ["MongoDB", "Beta"],
          ["In-memory", "Dev only"],
        ],
      },
      {
        type: "list",
        kicker: "Storage & interfaces",
        head: "How engines and people reach it",
        rows: [
          "Warehouses on <k>S3</k>, <k>Azure Blob</k> and <k>GCS</k>, configured per warehouse",
          "Iceberg REST at <k>/v1</k>, the management API at <k>/api/v1</k>",
          "A SvelteKit management UI, and <k>PyPangolin</k> on PyPI",
          "Everything versioned together at 0.6.0 — five numbers had drifted apart",
        ],
      },
      {
        type: "statement",
        kicker: "The substrate",
        big: "<u>unsafe_code = \"forbid\"</u> across the workspace.",
        sub: "334 tests run on every push and pull request, alongside fmt, clippy, cargo audit, helm lint and a Docker build that asserts the image is not root.",
      },
      {
        type: "close",
        big: "The full parity matrix is in <u>docs/operations</u>.",
        repo: "github.com/AlexMercedCoder/Pangolin",
      },
    ],
  },

  {
    slug: "whats-new-in-0-6-0",
    title: "What's New in 0.6.0",
    series: "Pangolin // 03 — Release",
    ghost: "0.6.0",
    scenes: [
      {
        type: "open",
        wordmark: "<u>0.6.0</u>",
        tagline: "This is a security release. If you run anything earlier, upgrade and rotate credentials.",
        specs: [
          ["Released", "2026-08-09"],
          ["Severity", "Critical"],
          ["Action", "Upgrade"],
        ],
      },
      {
        type: "list",
        kicker: "Fixed — exploitable",
        head: "Four ways in that no longer exist",
        rows: [
          "<k>A-8</k> The OAuth callback put the session token in an attacker-chosen redirect <q>— account takeover</q>",
          "<k>A-10</k> A working default JWT secret, published in this repository",
          "<k>A-11</k> Auth bypass: the whitelist matched any path <q>ending in</q> /config",
          "<k>A-12</k> One bogus API key burned ~25 CPU-seconds <q>— unauthenticated DoS</q>",
        ],
      },
      {
        type: "list",
        kicker: "Fixed — Iceberg correctness",
        head: "Silent data damage, now surfaced",
        rows: [
          "<k>assert-ref-snapshot-id</k> was never enforced — a losing writer re-applied its snapshot and forked the lineage, with no error",
          "Eleven commit update types were discarded while returning <k>200 OK</k>",
          "<k>last_sequence_number</k> was being assigned a random snapshot ID",
        ],
      },
      {
        type: "list",
        kicker: "Added",
        head: "You can finally operate it",
        rows: [
          "Prometheus metrics, request IDs, a working <k>RUST_LOG</k>, real health probes",
          "Graceful shutdown, plus body, timeout and concurrency limits <q>— there were none</q>",
          "A Helm chart whose three referenced-but-missing templates now exist",
          "CI on every push: build, fmt, clippy, test, audit, helm, docker",
        ],
      },
      {
        type: "statement",
        kicker: "Upgrading is not free",
        big: "<u>PANGOLIN_JWT_SECRET</u> is now required.",
        sub: "The server refuses to start without a real one. OAuth clients move to a code exchange, service-user API keys should be rotated, and probes move to /health/live and /health/ready.",
      },
      {
        type: "close",
        big: "Running 0.5.x? <u>Upgrade and rotate.</u>",
        repo: "SECURITY.md has the full advisory",
      },
    ],
  },

  {
    slug: "production-readiness",
    title: "Production Readiness",
    series: "Pangolin // 04 — Status",
    ghost: "status",
    scenes: [
      {
        type: "open",
        wordmark: "HOW <u>READY?</u>",
        tagline: "The maturity table, without the marketing. Pangolin previously claimed 100% Iceberg spec compliance. It does not now.",
        specs: [
          ["Version", "0.6.0"],
          ["Status", "Alpha"],
          ["Pre-1.0", "Yes"],
        ],
      },
      {
        type: "table",
        kicker: "Ready to rely on",
        head: "What 0.6.0 actually holds up",
        rows: [
          ["Iceberg table commits", "Solid"],
          ["Multi-tenancy and isolation", "Solid"],
          ["PostgreSQL backend", "Good"],
          ["Branching, tags and merge", "Good"],
          ["Observability", "New"],
        ],
      },
      {
        type: "table",
        kicker: "Not there yet",
        head: "What is still missing",
        rows: [
          ["Rate limiting", "Missing"],
          ["Full OIDC — PKCE, JWKS", "Missing"],
          ["Credential encryption at rest", "Missing"],
          ["Backup, restore and DR", "Untested"],
          ["MongoDB backend", "Beta"],
        ],
      },
      {
        type: "list",
        kicker: "Stated plainly",
        head: "The limitations worth knowing",
        rows: [
          "No per-IP or per-account throttle — <k>the login endpoint is brute-forceable</k>",
          "Branch creation by copy is <k>still not atomic</k>; take a backup first",
          "Warehouse cloud credentials sit <k>unencrypted</k> in the catalog database",
          "Missing endpoints: <k>registerTable</k>, <k>commitTransaction</k>, most of the view API",
        ],
      },
      {
        type: "statement",
        kicker: "So, should you?",
        big: "Evaluate it. <u>Don't bet the lake on it.</u>",
        sub: "Yes for evaluation, branching over catalog metadata, or contributing. Not yet if you need tested recovery, encrypted credentials, or full spec coverage — use Apache Polaris for that today.",
      },
      {
        type: "close",
        big: "The maturity table ships in the <u>README</u>.",
        repo: "github.com/AlexMercedCoder/Pangolin",
      },
    ],
  },

  {
    slug: "getting-started",
    title: "Getting Started",
    series: "Pangolin // 05 — Deploy",
    ghost: "deploy",
    scenes: [
      {
        type: "open",
        wordmark: "GET <u>STARTED</u>",
        tagline: "From cargo run to a Helm chart — and the secrets 0.6.0 will not start without.",
        specs: [
          ["Rust", "1.92+"],
          ["Chart", "0.6.0"],
          ["Images", "0.6.0"],
        ],
      },
      {
        type: "code",
        kicker: "Locally",
        head: "A clean clone should just be green",
        lines: [
          ["cmt", "# a Rust toolchain is the only dependency"],
          ["", "cd pangolin"],
          ["", "cargo test --workspace"],
          ["", "cargo run --bin pangolin_api"],
        ],
      },
      {
        type: "code",
        kicker: "Required in 0.6.0",
        head: "Every insecure default is gone",
        lines: [
          ["key", "PANGOLIN_JWT_SECRET=$(openssl rand -base64 48)"],
          ["key", "PANGOLIN_ADMIN_PASSWORD=..."],
          ["val", "PANGOLIN_STORAGE_TYPE=postgres"],
          ["val", "DATABASE_URL=postgres://..."],
          ["cmt", "# the server refuses to start without a real secret"],
        ],
      },
      {
        type: "list",
        kicker: "Or take the shortcut",
        head: "A demo stack in one command",
        rows: [
          "<k>docker compose up -d</k> in deployment_assets/demo/evaluate_single_tenant",
          "UI on <k>:3000</k>, API on <k>:8080</k>, Jupyter on <k>:8888</k>, MinIO on <k>:9001</k>",
          "A multi-tenant variant with JWT auth and SQLite sits beside it",
          "Production compose files for S3, Azure and GCS with Postgres or Mongo",
        ],
      },
      {
        type: "list",
        kicker: "Kubernetes",
        head: "The chart was repaired in this release",
        rows: [
          "Helm chart at <k>0.6.0</k>, with the ingress, HPA and ServiceAccount templates it referenced but never shipped",
          "Non-root, read-only root filesystem, all capabilities dropped",
          "Probes at <k>/health/live</k> and <k>/health/ready</k>, plus a ServiceMonitor",
          "It will not render without a real <k>PANGOLIN_JWT_SECRET</k>",
        ],
      },
      {
        type: "close",
        big: "Roll <u>one replica</u> first.",
        repo: "docs/operations/runbook.md",
      },
    ],
  },
];
