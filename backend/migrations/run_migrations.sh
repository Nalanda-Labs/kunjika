#!/usr/bin/env bash
# Apply the SQL migrations in backend/migrations/ in version order.
#
# Thin wrapper around run_migrations.py (the single implementation). Kept so
# existing muscle memory (./run_migrations.sh --status) keeps working; all
# flags are passed through verbatim. The Python script talks to YugabyteDB
# via psycopg (pip install "psycopg[binary]") - no psql needed.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec python3 "$here/run_migrations.py" "$@"
