# Backup and recovery

C-16. This page exists because the previous documentation described a backup
procedure that, as far as the repository showed, nobody had ever restored from.
A backup procedure that has not been executed is a hypothesis.

`scripts/backup_restore_drill.sh` runs the whole cycle — dump, **destroy**,
restore, verify — against a real database and fails loudly if anything does not
come back. Run it before you need it.

## What must be backed up

Two things, and losing either one loses your catalog.

| What | How | If you lose it |
|---|---|---|
| The catalog database | `pg_dump` | Everything: catalogs, tables, branches, users, permissions, audit history |
| `PANGOLIN_ENCRYPTION_KEY` | Your secret manager | The database restores fine and **every warehouse credential is unreadable** |

The second one is the trap. The key is not in the dump — it is deliberately not
in the dump — so a team that backs up the database religiously and never records
the key has a restore that produces a working catalog full of warehouses nobody
can authenticate to. Back the key up wherever you keep break-glass secrets, and
verify you can actually retrieve it.

`PANGOLIN_JWT_SECRET` is worth recording too, though losing it is milder: every
session is invalidated and users log in again.

Object storage is not backed up by any of this. Pangolin stores *pointers* to
Iceberg metadata; the metadata and data files live in your bucket and are
covered by whatever versioning and retention you have configured there.

## Taking a backup

```bash
pg_dump --format=custom --no-owner --no-privileges \
  --file=pangolin-$(date +%Y%m%d-%H%M%S).dump "$DATABASE_URL"
```

Use a `pg_dump` whose major version is **at least** the server's. An older
client refuses, and it refuses after you have already decided a backup exists.
The drill script checks this up front for that reason.

## Restoring

```bash
psql "$DATABASE_URL" -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;"
pg_restore --dbname="$DATABASE_URL" --no-owner --no-privileges \
  --exit-on-error pangolin-20260811-120000.dump
```

`--exit-on-error` matters. Without it `pg_restore` reports success while
skipping objects it could not create, which is exactly the failure a restore
drill is looking for.

Then start the server and check `/health/ready`, which probes the store rather
than just answering `200`.

## Verifying a restore

The drill checks three things beyond "the command exited zero":

1. **A canary row survives.** Row counts alone would compare equal if the
   restore silently produced an empty database.
2. **Row counts match** across catalogs, warehouses and assets.
3. **`_sqlx_migrations` is populated.** If the migration ledger does not
   restore, the next server start tries to re-run every migration against a
   database that already has the objects, and fails with
   `relation "tenants" already exists`. This is not hypothetical — it happened
   during development when migrations were applied by hand.

## Measured figures

From an actual run of the drill on a developer laptop, against PostgreSQL 15 in
Docker with 1,345 catalog/warehouse/asset rows:

| Phase | Time |
|---|---|
| Backup | 7s |
| Restore | 53s |
| **Mechanical RTO** | **~60s** |

**These numbers describe a laptop, not your production hardware**, and a dataset
that is small. Treat the shape as useful — restore dominates backup by roughly
8× — and re-measure on your own infrastructure. Run the drill against a copy of
production-sized data if you want a figure you can put in an SLA.

Your real RTO is that 60 seconds plus how long it takes to notice, decide, and
get someone with credentials to a terminal. That is usually the dominant term
and it is not something a script can measure.

**RPO is your dump schedule and nothing else.** Pangolin has no continuous
archiving or point-in-time recovery of its own. If you dump nightly, your RPO is
24 hours. If you need better, use PostgreSQL WAL archiving or your cloud
provider's managed backups — both are outside Pangolin and both work fine with
it.

## Running the drill

```bash
DATABASE_URL=postgres://user:pass@host/pangolin_test scripts/backup_restore_drill.sh
```

It refuses to run against a database whose name does not contain `test`,
`drill`, `scratch` or `tmp`, because it drops the schema. That guard is
deliberate and you should not remove it; point it at a restored copy of
production instead, which also gives you a more honest number.

## What is not covered

- **No point-in-time recovery.** Dump-and-restore only.
- **No automated backup scheduling.** Use cron, a Kubernetes CronJob, or your
  provider's managed backups.
- **No tested MongoDB or SQLite drill.** The script is PostgreSQL-only.
  `mongodump`/`mongorestore` and copying the SQLite file are the equivalents,
  but neither has been drilled here, so neither is written up as though it had
  been.
- **Multi-region failover is out of scope.** This covers restoring one database.
