-- Business metadata for assets: descriptions, tags, discoverability.
--
-- This table was never created on Postgres. `search_assets` has always run
-- `LEFT JOIN business_metadata m ON a.id = m.asset_id`, so *every* asset search
-- on the Postgres backend failed outright with
-- `relation "business_metadata" does not exist` - not an empty result, a hard
-- SQL error. The three CRUD methods were not implemented either, so the trait's
-- "Operation not supported by this store" default answered them.
--
-- Found by running the cross-backend parity suite against a live Postgres for
-- the first time. SQLite, Mongo and the memory backend all had it.
--
-- Column types follow the Postgres conventions already in this schema: UUID for
-- ids, JSONB for structured values, TIMESTAMPTZ for times - rather than the
-- TEXT/INTEGER encoding SQLite uses.

CREATE TABLE IF NOT EXISTS business_metadata (
    id           UUID PRIMARY KEY,
    asset_id     UUID NOT NULL UNIQUE,
    description  TEXT,
    tags         JSONB NOT NULL DEFAULT '[]'::jsonb,
    properties   JSONB NOT NULL DEFAULT '{}'::jsonb,
    discoverable BOOLEAN NOT NULL DEFAULT FALSE,
    created_by   UUID NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by   UUID NOT NULL,
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_business_metadata_asset
        FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE CASCADE
);

-- `search_assets` joins on asset_id and filters on description; the unique
-- constraint above already indexes asset_id.
CREATE INDEX IF NOT EXISTS idx_business_metadata_discoverable
    ON business_metadata(discoverable)
    WHERE discoverable;

-- Tag filtering uses the containment operator (`tags @> $n::jsonb`), which GIN
-- serves directly.
CREATE INDEX IF NOT EXISTS idx_business_metadata_tags
    ON business_metadata USING GIN (tags);
