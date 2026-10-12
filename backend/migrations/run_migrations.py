#!/usr/bin/env python3
"""Apply the SQL migrations in backend/migrations/ in version order.

Why this exists: the backend has no migrator wired up. Nothing in
backend/src/ calls sqlx::migrate!(), and the database has no
_sqlx_migrations table, so migrations were being applied by hand. That makes
it easy to skip one or run them out of order. This applies every
<version>_<description>.sql in this directory, in order, each in its own
transaction, and records what it applied.

Order is the numeric filename prefix, which is a UTC timestamp
(20220620050241, 20261001160000, ...), so sorting by version is sorting by
time. Files that do not match <version>_<description>.sql are ignored, which
is the same rule sqlx::migrate!() uses -- so keeping this script here does not
confuse a future migrator.

Usage:
    ./run_migrations.py             apply pending migrations
    ./run_migrations.py --status    list applied/pending, change nothing
    ./run_migrations.py --dry-run   show what would be applied
    ./run_migrations.py --force     re-apply already-applied migrations
    ./run_migrations.py --baseline  record pending migrations WITHOUT running
                                    them (use once, to adopt this script on a
                                    database whose schema is already current)

Target database: $DATABASE_URL, falling back to backend/.env (the same file
sqlx reads for its compile-time query checking). Override inline:
    DATABASE_URL=postgres://user:pw@127.0.0.1:5433/db ./run_migrations.py

Driver: psycopg (v3) over a direct TCP connection. Install it with:
    pip install "psycopg[binary]"
(the binary wheel is self-contained: no libpq, no postgresql-client, no
psql needed - deliberate, since this project targets YugabyteDB, not
vanilla PostgreSQL).

Requires psycopg 3.x (`import psycopg`).
"""

import hashlib
import os
import re
import subprocess
import sys
from pathlib import Path
from urllib.parse import urlparse

try:
    import psycopg
except ImportError:
    print(
        "error: the 'psycopg' package (v3) is required.\n"
        "       install it with: pip install \"psycopg[binary]\"\n"
        "       (self-contained wheel - no libpq / postgresql-client / psql).",
        file=sys.stderr,
    )
    sys.exit(1)

HERE = Path(__file__).resolve().parent
BACKEND = HERE.parent
TABLE = "kunjika_migrations"

VERSION_RE = re.compile(r"^([0-9]+)_(.+)\.sql$")


def fail(msg):
    print(f"error: {msg}", file=sys.stderr)
    sys.exit(1)


def read_database_url():
    url = os.environ.get("DATABASE_URL", "").strip().strip("\"'")
    if not url:
        env_file = BACKEND / ".env"
        if env_file.is_file():
            for line in env_file.read_text().splitlines():
                m = re.match(r"\s*DATABASE_URL\s*=\s*(.*)", line)
                if m:
                    url = m.group(1).strip().strip("\"'")
        if not url:
            fail(
                "no DATABASE_URL set and none found in "
                f"{BACKEND / '.env'}\n"
                "       set it, e.g. DATABASE_URL=postgres://user:pw@host:5433/db \\\n"
                f"       {Path(sys.argv[0]).name}"
            )
    return url


def connect(url):
    """Open a psycopg connection (autocommit off; we manage transactions)."""
    try:
        return psycopg.connect(url)
    except Exception as e:
        fail(f"could not connect to {redact(url)}: {e}")


def redact(url):
    try:
        p = urlparse(url)
        host = p.hostname or ""
        port = f":{p.port}" if p.port else ""
        db = (p.path or "").lstrip("/")
        return f"{p.scheme}://{p.username or ''}@***{host}{port}/{db}"
    except Exception:
        return "<database>"


def detect_engine(conn):
    with conn.cursor() as cur:
        cur.execute("select version();")
        out = str(cur.fetchone()[0])
    return "yugabyte" if "-YB-" in out else "postgres"


def ensure_table(conn):
    with conn.cursor() as cur:
        cur.execute(
            f"CREATE TABLE IF NOT EXISTS {TABLE} (\n"
            "  version     bigint primary key,\n"
            "  description text not null,\n"
            "  checksum    text not null,\n"
            "  applied_at  timestamptz not null default now()\n"
            ");"
        )
    conn.commit()


def applied_rows(conn):
    with conn.cursor() as cur:
        try:
            cur.execute(f"SELECT version, checksum FROM {TABLE};")
            rows = cur.fetchall()
        except Exception:
            # Table did not exist a moment ago (first run) or the user
            # lacks rights - either way, treat everything as pending; the
            # apply path surfaces the real error.
            conn.rollback()
            return {}
    conn.commit()
    seen = {}
    for version, checksum in rows:
        try:
            seen[int(version)] = str(checksum).strip()
        except (TypeError, ValueError):
            continue
    return seen


def discover():
    found = []
    for path in sorted(HERE.glob("[0-9]*.sql")):
        m = VERSION_RE.match(path.name)
        if not m:
            continue
        version = int(m.group(1))
        desc = path.name.split("_", 1)[1][:-len(".sql")]
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        found.append((version, desc, digest, path))
    found.sort(key=lambda row: row[0])
    return found


def main(argv):
    mode = "apply"
    if len(argv) > 1:
        opt = argv[1]
        modes = {
            "-h": "help", "--help": "help",
            "--status": "status", "--dry-run": "dry-run",
            "--force": "force", "--baseline": "baseline",
        }
        if opt not in modes:
            fail(f"unknown option {opt!r} (try --help)")
        mode = modes[opt]
    if mode == "help":
        print(__doc__.strip())
        return 0

    db_url = read_database_url()
    conn = connect(db_url)
    engine = detect_engine(conn)
    ensure_table(conn)

    seen = applied_rows(conn)
    if mode == "force":
        seen = {}
    migrations = discover()
    pending = []
    for version, desc, digest, path in migrations:
        if version in seen and seen[version] == digest:
            continue
        if version in seen:
            print(
                f"warning: {version}_{desc} changed after it was applied; "
                "not re-running.\n"
                "         If that edit is intentional, re-run with --force.",
                file=sys.stderr,
            )
            continue
        pending.append((version, desc, digest, path))

    if mode == "status":
        print(f"{'VERSION':<16} {'STATUS':<10} DESCRIPTION")
        for version, desc, _, _ in migrations:
            status = "applied" if version in seen else "PENDING"
            print(f"{version:<16} {status:<10} {desc}")
        conn.close()
        return 0

    if not pending:
        print("up to date, nothing to apply.")
        conn.close()
        return 0

    print()
    if mode == "dry-run":
        print(f"would apply {len(pending)} migration(s):")
        for version, desc, _, _ in pending:
            print(f"  {version:<16} {desc}")
        conn.close()
        return 0

    if mode == "baseline":
        # Record without executing. Each row commits on its own so a single
        # bad one cannot roll back the rest, and we do not touch schema.
        with conn.cursor() as cur:
            for version, desc, digest, _ in pending:
                cur.execute(
                    f"INSERT INTO {TABLE} (version, description, checksum)\n"
                    "VALUES (%s, %s, %s)\n"
                    "ON CONFLICT (version) DO UPDATE\n"
                    "  SET checksum = EXCLUDED.checksum, applied_at = now();",
                    (version, desc, digest),
                )
                conn.commit()
                print(f"  baselined {version}  {desc}")
        print()
        print(f"baselined {len(pending)} migration(s) without running them.")
        print("NOTE: the schema was NOT changed. Verify it already matches.")
        conn.close()
        return 0

    applied_n = 0
    for version, desc, digest, path in pending:
        # The file body goes through `psql -f` (present on bare metal): it
        # handles multi-statement pg_dump output, backslash commands and
        # per-statement error reporting, which psycopg's
        # execute-one-string path cannot do ("cannot insert multiple commands
        # into a prepared statement"). The bookkeeping INSERT stays in
        # psycopg with proper parameters, committed only after psql succeeds.
        file_rc = subprocess.run(
            [
                "psql", db_url,
                "-v", "ON_ERROR_STOP=1",
                "-f", str(path),
            ],
            capture_output=True, text=True,
        )
        if file_rc.returncode != 0:
            print(f"  FAILED   {version}  {desc}", file=sys.stderr)
            tail = (file_rc.stderr or file_rc.stdout or "").strip()
            if tail:
                print(f"  psql: {tail.splitlines()[-1]}", file=sys.stderr)
            print(file=sys.stderr)
            print(
                f"not recorded in {TABLE}, so it will be retried on the next run.",
                file=sys.stderr,
            )
            if engine == "yugabyte":
                print(
                    "WARNING: YugabyteDB does not roll back DDL, so every "
                    f"statement in\n         {path.name} up to the failure is "
                    "still in effect.\n"
                    "         Make the migration idempotent "
                    "(IF NOT EXISTS / OR REPLACE)\n"
                    "         or undo the partial changes by hand, then re-run.",
                    file=sys.stderr,
                )
            else:
                print(
                    "psql stopped at the first error (ON_ERROR_STOP), so no "
                    "later statements ran.",
                    file=sys.stderr,
                )
            print("no further migrations were attempted.", file=sys.stderr)
            conn.close()
            return 1
        try:
            with conn.cursor() as cur:
                cur.execute(
                    f"INSERT INTO {TABLE} (version, description, checksum)\n"
                    "VALUES (%s, %s, %s);",
                    (version, desc, digest),
                )
            conn.commit()
            print(f"  applied  {version}  {desc}")
            applied_n += 1
            continue
        except Exception as e:
            try:
                conn.rollback()
            except Exception:
                conn.close()
                conn = connect(db_url)
            print(f"  FAILED   {version}  {desc}", file=sys.stderr)
            print(f"  bookkeeping INSERT failed: {type(e).__name__}: {e}",
                  file=sys.stderr)
            print(
                f"the schema change in {path.name} is applied, but it is NOT "
                f"recorded in {TABLE} - fix the bookkeeping table and re-run "
                "with --force.",
                file=sys.stderr,
            )
            print("no further migrations were attempted.", file=sys.stderr)
            conn.close()
            return 1

    conn.close()
    print()
    print(f"applied {applied_n} migration(s).")
    if applied_n > 0:
        print(
            "Restart the backend: sqlx caches prepared statements, so a DDL "
            "change\n"
            "can otherwise fail with 0A000 until the process is recycled."
        )
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
