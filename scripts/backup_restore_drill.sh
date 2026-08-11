#!/usr/bin/env bash
#
# Back up a Pangolin catalog database, destroy it, restore it, and verify the
# catalog is intact.
#
# C-16. There was a backup section in the documentation and no evidence anyone
# had ever restored from one. A backup procedure that has not been executed is a
# hypothesis, not a recovery plan: the failure modes are things like the dump
# omitting the schema, the restore running against a database that already has
# objects, or - the one that actually bites - the encryption key not being
# backed up alongside the data, so the rows restore fine and every warehouse
# credential is unreadable.
#
# This drills the whole cycle against a real database and fails loudly if the
# restored catalog does not match what was backed up.
#
# Usage:
#   scripts/backup_restore_drill.sh                 # against PANGOLIN_TEST_POSTGRES_URL
#   DATABASE_URL=postgres://... scripts/backup_restore_drill.sh
#
# Requires: pg_dump, psql, and a PostgreSQL the script may DROP SCHEMA on.
# It refuses to run against a database whose name does not look disposable.

set -euo pipefail

DB_URL="${DATABASE_URL:-${PANGOLIN_TEST_POSTGRES_URL:-}}"
if [[ -z "$DB_URL" ]]; then
    echo "error: set DATABASE_URL or PANGOLIN_TEST_POSTGRES_URL" >&2
    exit 1
fi

# This script drops the public schema. Refuse anything that does not look like a
# throwaway database, because the cost of being wrong is the whole catalog.
if [[ ! "$DB_URL" =~ (test|drill|scratch|tmp) ]]; then
    echo "error: refusing to run against '$DB_URL'." >&2
    echo "       This script DROPS SCHEMA public. Point it at a disposable" >&2
    echo "       database whose name contains test, drill, scratch or tmp." >&2
    exit 1
fi

# A pg_dump older than the server refuses to run, and it refuses *after* you
# have decided a backup exists. Check first, with a message that names the fix.
# No `| head -1` here: `head` closes the pipe as soon as it has its line, the
# upstream command takes SIGPIPE, and `set -o pipefail` turns that into a
# non-zero status that `set -e` acts on. It raced on one machine and failed
# every time inside the postgres container - the whole reason to run a drill
# rather than trust the procedure.
server_raw=$(psql "$DB_URL" -Atc "SHOW server_version;")
server_version=${server_raw%%.*}
client_raw=$(pg_dump --version)      # "pg_dump (PostgreSQL) 15.15"
client_version=${client_raw##* }     # -> "15.15"
client_version=${client_version%%.*} # -> "15"
if [[ "$client_version" -lt "$server_version" ]]; then
    cat >&2 <<MSG
error: pg_dump is version $client_version and the server is $server_version.
       pg_dump refuses to dump a newer server, so this would fail partway.

       Use a client matching the server. If the server runs in Docker:

         docker exec <container> pg_dump ... > dump.custom

       On Debian/Ubuntu, postgresql-client-$server_version provides a matching
       pg_dump under /usr/lib/postgresql/$server_version/bin.
MSG
    exit 1
fi

WORK_DIR="$(mktemp -d)"
DUMP="$WORK_DIR/pangolin.dump"
trap 'rm -rf "$WORK_DIR"' EXIT

step() { printf '\n=== %s ===\n' "$1"; }
fail() { echo "FAIL: $1" >&2; exit 1; }

step "Recording the pre-backup state"
# `-Atc` gives an unadorned value, so these compare cleanly.
count_before=$(psql "$DB_URL" -Atc "
    SELECT COALESCE(
        (SELECT count(*) FROM catalogs), 0
    ) + COALESCE(
        (SELECT count(*) FROM warehouses), 0
    ) + COALESCE(
        (SELECT count(*) FROM assets), 0
    );" 2>/dev/null || echo "0")
echo "catalogs + warehouses + assets: $count_before"

if [[ "$count_before" == "0" ]]; then
    echo "note: the database is empty, so this drill proves the mechanics but"
    echo "      not that real data survives. Seed it first for a meaningful run."
fi

# A canary. If the restore silently produces an empty database, a row count of
# zero on both sides would still 'match'.
step "Planting a canary row"
psql "$DB_URL" -q -c "
    CREATE TABLE IF NOT EXISTS _drill_canary (id text primary key, planted_at timestamptz);
    INSERT INTO _drill_canary (id, planted_at) VALUES ('canary', now())
    ON CONFLICT (id) DO UPDATE SET planted_at = now();" \
    || fail "could not plant the canary"

step "Backing up"
start=$(date +%s)
pg_dump --format=custom --no-owner --no-privileges --file="$DUMP" "$DB_URL" \
    || fail "pg_dump failed"
backup_secs=$(( $(date +%s) - start ))
echo "wrote $(du -h "$DUMP" | cut -f1) in ${backup_secs}s"

step "Destroying the database"
# This is the part a drill exists to do. A backup nobody has restored from is
# not a backup.
psql "$DB_URL" -q -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;" \
    || fail "could not drop the schema"

remaining=$(psql "$DB_URL" -Atc \
    "SELECT count(*) FROM information_schema.tables WHERE table_schema='public';")
[[ "$remaining" == "0" ]] || fail "the schema was not actually dropped ($remaining tables left)"
echo "schema dropped"

step "Restoring"
start=$(date +%s)
# --exit-on-error, because a restore that reports success while skipping half
# the objects is the failure mode this drill is looking for.
pg_restore --dbname="$DB_URL" --no-owner --no-privileges --exit-on-error "$DUMP" \
    || fail "pg_restore failed"
restore_secs=$(( $(date +%s) - start ))
echo "restored in ${restore_secs}s"

step "Verifying"
canary=$(psql "$DB_URL" -Atc "SELECT id FROM _drill_canary WHERE id='canary';" 2>/dev/null || echo "")
[[ "$canary" == "canary" ]] || fail "the canary row did not survive the restore"
echo "canary present"

count_after=$(psql "$DB_URL" -Atc "
    SELECT COALESCE(
        (SELECT count(*) FROM catalogs), 0
    ) + COALESCE(
        (SELECT count(*) FROM warehouses), 0
    ) + COALESCE(
        (SELECT count(*) FROM assets), 0
    );" 2>/dev/null || echo "0")

if [[ "$count_before" != "$count_after" ]]; then
    fail "row counts differ: $count_before before, $count_after after"
fi
echo "catalogs + warehouses + assets: $count_after (unchanged)"

# The migration table matters: if it did not restore, the next server start
# reapplies every migration against a populated database and fails.
migrations=$(psql "$DB_URL" -Atc \
    "SELECT count(*) FROM _sqlx_migrations;" 2>/dev/null || echo "0")
[[ "$migrations" != "0" ]] || fail "_sqlx_migrations is empty; the next startup will try to re-run every migration"
echo "_sqlx_migrations: $migrations rows"

psql "$DB_URL" -q -c "DROP TABLE IF EXISTS _drill_canary;" || true

cat <<EOF

=== Result ===
Backup:   ${backup_secs}s
Restore:  ${restore_secs}s
RTO for this dataset: ~$(( backup_secs + restore_secs ))s of mechanical work,
plus however long it takes you to notice and decide.

RPO is whatever your dump schedule is. This script takes a dump now; it says
nothing about how old your most recent real backup is.

Reminder: PANGOLIN_ENCRYPTION_KEY is NOT in this dump. Restoring the database
without it leaves every warehouse credential unreadable. Back the key up
separately and verify you can retrieve it.
EOF
