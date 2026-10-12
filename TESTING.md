# Testing Kunjika

Test suites for the ntex/Yugabyte backend and the SvelteKit
frontend.

## Current status

| Suite | Command | Result |
| --- | --- | --- |
| Backend REST (`cargo test`) | [section 2](#2-backend-test-harness) — STARTTLS env vars required | **32 passed** |
| Backend unit (`cargo test --bin kunjika`) | `cd backend && cargo test --bin kunjika` | **3 passed** |
| Frontend e2e (Playwright) | `cd frontend && npx playwright test -c playwright.e2e.config.ts` | **13 passed** |

---

## 1. Containers (rootless podman)

The app's migration uses YugabyteDB-flavoured DDL (`USING lsm` indexes,
`citext`), so the database must be YugabyteDB, not vanilla PostgreSQL.

```bash
podman network create kunjika-test

podman run -d --name kunjika-yb --network kunjika-test \
  -p 5433:5433 -p 7000:7000 -p 9000:9000 \
  docker.io/yugabytedb/yugabyte:latest

podman run -d --name kunjika-redis --network kunjika-test \
  -p 6379:6379 docker.io/library/redis:7-alpine

# Plaintext SMTP on 2525, API on 8025 — optional control instance.
# The tracked test config (template.test.json) uses the STARTTLS
# instance from section 1b instead.
podman run -d --name kunjika-mailpit-plain --network kunjika-test \
  -p 2525:1025 -p 8025:8025 docker.io/axllent/mailpit:latest
```

Wait for YSQL, then create the database:

```bash
until podman exec kunjika-yb bash -c \
  "bin/ysqlsh -h \$(hostname -i) -U yugabyte -d yugabyte -c 'select 1'" 2>/dev/null; do sleep 5; done

podman exec kunjika-yb bash -c \
  "bin/ysqlsh -h \$(hostname -i) -U yugabyte -d yugabyte -c 'CREATE DATABASE kunjika_test'"
```

Then apply the migrations with the helper script (see [section 5](#5-applying-migrations)):

```bash
cd backend/migrations && ./run_migrations.py --status && ./run_migrations.py
```

This builds the schema from empty: all 8 tables plus sequences, indexes, views
and triggers, with `username` already widened to 33 characters and the expiry
triggers attached to the right tables. On a database that is already at the
current schema, adopt the script with `--baseline` instead of re-running
anything.

Verify the widened column (note: `information_schema` can briefly report a
stale value on YugabyteDB, so confirm with `\d users` or by inserting an
11-character username):

```bash
podman exec kunjika-yb bash -c \
  "bin/ysqlsh -h \$(hostname -i) -U yugabyte -d kunjika_test -c '\d users'"
# username | character varying(33) | not null |
# "users_name_key" UNIQUE CONSTRAINT, lsm (username HASH)   <- index rebuilt
```

> **Restart the backend after applying this migration.** sqlx caches prepared
> statements per connection, and YugabyteDB rejects a cached plan whose result
> type changed:
>
> ```
> ERROR 0A000: cached plan must not change result type
> ```
>
> Every `SELECT ... FROM users` then fails and `/auth/login` answers 401.
> Restarting the process (or recycling the pool) rebuilds the cache. This
> matters in production too: schedule a restart with the migration.

### 1b. STARTTLS mail server (the path the backend uses)

The backend's mailer is configured entirely from the config file
(`backend/src/config.rs`):

| Key | Meaning |
| --- | --- |
| `mail_host` / `mail_port` | SMTP relay address (may be a container/other host) |
| `mail_starttls` | `true`: upgrade to TLS via STARTTLS before sending — **required**, no plaintext fallback. `false`: plaintext (e.g. a trusted local relay) |
| `mail_tls_ca` | Optional PEM file with extra CA/root certs to trust for the handshake (e.g. a self-signed cert). Relative paths resolve against the config file's directory. Empty = system trust store only. Ignored when `mail_starttls` is `false` |

If `mail_username` is non-empty, SMTP `Credentials` are attached too.

`backend/template.test.json` (the config the harness spawns the backend
with) is pre-set for the TLS-only instance below:

```json
"mail_host": "127.0.0.1",
"mail_port": 1026,
"mail_starttls": true,
"mail_tls_ca": "test-harness/certs/mailpit-cert.pem",
```

The test certificate lives in `backend/test-harness/certs/` —
self-signed, 30-day validity, regenerate when expired:

```bash
openssl req -x509 -newkey rsa:2048 -nodes -days 30 \
  -keyout backend/test-harness/certs/mailpit-key.pem \
  -out    backend/test-harness/certs/mailpit-cert.pem \
  -subj "/CN=127.0.0.1" -addext "subjectAltName=IP:127.0.0.1,DNS:localhost"
```

The cert must carry an IP SAN for whatever `mail_host` resolves to (the
harness connects to `127.0.0.1`); it is then trusted for the handshake
via `mail_tls_ca`. Mailpit enforcing STARTTLS, on its own ports so it can
run next to the plaintext instance above (SMTP `:1026`, API/UI `:8026`):

```bash
podman run -d --name kunjika-mailpit-tls --network kunjika-test \
  -p 1026:1025 -p 8026:8025 \
  -v "$PWD/backend/test-harness/certs/mailpit-cert.pem:/tmp/smtp-cert.pem:ro,Z" \
  -v "$PWD/backend/test-harness/certs/mailpit-key.pem:/tmp/smtp-key.pem:ro,Z" \
  docker.io/axllent/mailpit:latest \
  --smtp-tls-cert /tmp/smtp-cert.pem --smtp-tls-key /tmp/smtp-key.pem \
  --smtp-require-starttls

cd backend/test-harness && cargo test --test smtp_tls_probe   # 3 passed
```

The backend validates the whole mail path at boot: `into_state()` calls
`test_connection()`, which performs the full STARTTLS handshake
(including certificate verification against `mail_tls_ca`) and panics
with `Error connecting to mail server!` if it fails; a healthy startup
logs `Connection successful!`. The three `smtp_tls_probe` tests cover
the server directly: banner/EHLO advertise STARTTLS, the handshake
succeeds with the test CA, and plaintext mail is rejected. Check by hand:

```bash
openssl s_client -starttls smtp -connect 127.0.0.1:1026 \
  -CAfile backend/test-harness/certs/mailpit-cert.pem </dev/null
```

---

## 2. Backend test harness

`backend/test-harness/` is a **separate crate**, so `backend/Cargo.toml` never
needs a `[dev-dependencies]` entry. It drives the *running* backend over HTTP and
asserts against YSQL and Mailpit.

```bash
cd backend/test-harness
# STARTTLS mail path (tracked test config → SMTP :1026, API :8026):
KUNJIKA_TEST_SMTP_PORT=1026 KUNJIKA_TEST_MAILPIT_URL=http://127.0.0.1:8026 \
  cargo test -- --test-threads=1        # 32 tests
```

Those two env vars point the harness at the STARTTLS Mailpit instance
from [section 1b](#1b-starttls-mail-server-the-path-the-backend-uses):
`KUNJIKA_TEST_SMTP_PORT` feeds the `smtp_tls_probe` tests, and
`KUNJIKA_TEST_MAILPIT_URL` is where the `api_mail` tests poll for
delivered mail — the backend delivers to `:1026`, so polling the
plaintext instance on `:8025` finds nothing. To run against plaintext
Mailpit instead, set `mail_starttls: false` and `mail_port: 1025` in
`backend/template.test.json` and drop the env overrides (code defaults:
SMTP `2525`, API `http://127.0.0.1:8025`).

| File | Covers |
| --- | --- |
| `api_public.rs` | public read endpoints; anonymous writes rejected |
| `api_users_a..d.rs` | register happy path, validation, availability, OTP login, 3/33/34-char username boundaries, 23505-vs-other error mapping, email/tag uniqueness |
| `api_mail.rs` | confirmation mail, `confirm-email`, OTP mail + single-use OTP — all delivered over STARTTLS |
| `api_writes_a/b.rs` | question lifecycle, answer, edit, vote, bookmark, duplicate tags, edit ownership/validation, `reply_to` shapes, self-answer with no notification |
| `smtp_tls_probe.rs` | STARTTLS handshake against the TLS mail server (section 1b) |

### Why the harness can spawn the backend with an `LD_PRELOAD` shim

Historically `backend/src/config.rs` hard-coded
`AsyncSmtpTransport::unencrypted_localhost()` → `127.0.0.1:25`, ignoring
`Config::mail_port` (lettre's `unencrypted_localhost()` takes no port
argument), while `backend/src/main.rs` refuses to run as root, so nothing
could put a listener on `:25`. `native/smtp_port_redirect.c` interposes
`connect(2)` for the backend process only and rewrites destination port
25 → 2525. The harness compiles it automatically (`ensure_smtp_shim`)
and sets `LD_PRELOAD`, `KUNJIKA_SMTP_REDIRECT_FROM=25`,
`KUNJIKA_SMTP_REDIRECT_TO=2525`.

The mailer now honours `mail_host`/`mail_port`/`mail_starttls`/
`mail_tls_ca` from the config file
([section 1b](#1b-starttls-mail-server-the-path-the-backend-uses)), so with
the tracked `template.test.json` (STARTTLS on `:1026`) the shim is inert —
the backend never connects to `:25`. It remains useful for plaintext
setups that still expect a relay on `:25`. Set `KUNJIKA_TEST_NO_SHIM=1`
to skip compiling it.

### Running against an already-running backend

```bash
cd backend/test-harness
KUNJIKA_TEST_NO_SPAWN=1 KUNJIKA_TEST_NO_SHIM=1 \
KUNJIKA_TEST_SMTP_PORT=1026 KUNJIKA_TEST_MAILPIT_URL=http://127.0.0.1:8026 \
cargo test -- --test-threads=1
```

(`KUNJIKA_TEST_NO_SHIM` is optional here — with nothing listening on
`:25` the shim never fires.)

Start the backend yourself with — no shim needed, the mailer connects to
the STARTTLS instance on `:1026` directly:

```bash
cd backend
./target/debug/kunjika -c template.test.json -v 1
```

### Environment variables

| Variable | Default | STARTTLS run ([section 1b](#1b-starttls-mail-server-the-path-the-backend-uses)) |
| --- | --- | --- |
| `KUNJIKA_TEST_BASE_URL` | `http://127.0.0.1:8000/api/v1` | — |
| `KUNJIKA_TEST_DB_URL` | `postgres://yugabyte:yugabyte@127.0.0.1:5433/kunjika_test` | — |
| `KUNJIKA_TEST_MAILPIT_URL` | `http://127.0.0.1:8025` (plaintext API) | `http://127.0.0.1:8026` |
| `KUNJIKA_TEST_SMTP_PORT` | `2525` (plaintext, via the redirect shim) | `1026` |
| `KUNJIKA_TEST_SMTP_FROM_PORT` | `25` | — |
| `KUNJIKA_TEST_NO_SPAWN` | unset (harness spawns the backend) | `1` when a backend is already running |
| `KUNJIKA_TEST_NO_SHIM` | unset (port-25 shim compiled and preloaded) | `1` to skip the shim |
| `KUNJIKA_TEST_BACKEND_BIN` | `backend/target/debug/kunjika` | — |
| `KUNJIKA_TEST_CONFIG` | `backend/template.test.json` | — |

The mailer's TLS behaviour is not env-driven: it comes from the config
file (`mail_host`, `mail_port`, `mail_starttls`, `mail_tls_ca`) —
see [section 1b](#1b-starttls-mail-server-the-path-the-backend-uses).

### Regenerating the test JWT keys

`template.test.json` carries base64-encoded RSA PEMs; empty values make
`POST /auth/login` answer `502 {"message":"InvalidKeyFormat"}`.

```bash
./backend/test-harness/scripts/gen_test_keys.sh
```

---

## 3. Frontend e2e (Playwright)

```bash
cd frontend
npx playwright install chromium
npx playwright test -c playwright.e2e.config.ts
```

`playwright.e2e.config.ts` is additive; the repo's own `playwright.config.ts`
(which has no `baseURL`, so `page.goto('/')` cannot resolve) is untouched.

It starts `npm run build && npm run preview` on **port 5173** on purpose: the
backend CORS allow-list in `backend/src/main.rs` only accepts
`http://localhost:5173`, `http://localhost:3000` and `http://localhost`.

`auth.spec.ts` runs a genuine full-stack flow: register in the UI, read the
confirmation token out of Mailpit, `confirm-email`, request an OTP, read the OTP
from Mailpit, and finish the login in the browser — then assert the browser
actually stored the `access_token` cookie. `auth.spec.ts` polls Mailpit's
API on `http://localhost:8026` (the `MAILPIT` const), which is the API port of
the STARTTLS instance from [section 1b](#1b-starttls-mail-server-the-path-the-backend-uses),
so the Playwright flow works against a backend using the tracked
`template.test.json` as-is.

## 4. sqlx and `DATABASE_URL`

The `sqlx::query!` macros are checked at **compile** time, so `cargo build` and
`cargo test --bin kunjika` need a live database. sqlx loads `backend/.env`
automatically, so no environment variable is required as long as that file
points at a reachable database:

```
DATABASE_URL=postgres://yugabyte:yugabyte@127.0.0.1:5433/kunjika_test
```

`backend/.env` is gitignored, so it is a local file. It previously read
`postgres://shiv:yagyavalkya@127.0.1.1:5433/kunjika`, which could not work here:
the role `shiv` does not exist in the YugabyteDB container and the `kunjika`
database is never created. (`127.0.1.1` is *not* a typo for `127.0.0.1` in any
harmful sense - the whole of `127.0.0.0/8` is loopback on Linux, so it still
reached the container; the failure was the role and database name.)

Override per-invocation when you want to compile against something else:

```bash
DATABASE_URL=postgres://yugabyte:yugabyte@127.0.0.1:5433/kunjika_test cargo test --bin kunjika
```

---

## 5. Applying migrations

`backend/migrations/run_migrations.py` applies every
`<version>_<description>.sql` in order - each file body through `psql -f` with
`ON_ERROR_STOP=1` - and records what it applied (version, description,
checksum) in a `kunjika_migrations` table.

```bash
cd backend/migrations
./run_migrations.py --status     # what is applied / what is pending
./run_migrations.py              # apply the pending ones
```

It exists because **nothing in `backend/src/` runs migrations**: there is no
`sqlx::migrate!()` call and no `_sqlx_migrations` table, so schema changes were
being applied by hand. Keeping the script *inside* the migrations directory is
safe - `sqlx::migrate!()` only reads files matching
`<version>_<description>.sql`, so it ignores a `.py` file sitting next to them.

The target database is `$DATABASE_URL`, falling back to `backend/.env`. Two
requirements: the `psycopg` v3 driver for the connection and bookkeeping
(`pip install "psycopg[binary]"` - the wheel is self-contained, no libpq
needed), and `psql` on `PATH` to apply the file bodies, which are
multi-statement pg_dump output that psycopg cannot execute as a single
statement. Other modes: `--dry-run` (show what would be applied),
`--force` (re-apply already-applied migrations), and `--baseline` below.

`--baseline` records pending migrations **without running them**. Use it once on
a database whose schema is already current, such as the existing test DB:

```bash
./run_migrations.py --baseline
```

### Two YugabyteDB facts worth knowing

1. **DDL is not transactional.** A `CREATE TABLE` survives a transaction
   `ROLLBACK` (verified on this container). A failed migration therefore leaves
   partial schema behind, and the script says so explicitly rather than claiming
   a clean rollback. Write migrations idempotently (`IF NOT EXISTS`,
   `CREATE OR REPLACE`) so a retry is safe - a failed migration is never
   recorded, so it will be retried on the next run.
2. **The initial migration could not run on an empty database.** It is raw
   `pg_dump` output that attached two triggers before their tables existed:
   `expire_views_delete_old_rows_trigger ... ON public.views` sat at line 12
   while `CREATE TABLE public.views` was at line 265, and the same for
   `public.tokens` (line 24 vs line 202). Applying it to a fresh database
   failed with `relation "public.views" does not exist`. **Fixed**: all four
   triggers are now defined at the end of the file, after every `CREATE TABLE`.
   A fresh database now produces a schema object-for-object identical to
   `kunjika_test`, and the triggers were verified to fire (inserting a view row
   30 minutes old prunes it). The `CREATE FUNCTION` statements stayed at the
   top, which is correct: a plpgsql body is not resolved until the function
   runs.

   Because the file changed, a database that had already applied it will see a
   checksum-drift warning from `run_migrations.py`. That is the intended
   behaviour - re-baseline it once with `--baseline`, which only records the new
   checksum and does not re-run anything.

3. **The initial migration could not complete on a fresh database.** Four
   separate defects, all now fixed, and a virgin database is byte-for-byte
   identical to the long-standing `kunjika_test` one:

   | Was | Problem | Fix |
   | --- | --- | --- |
   | 2 errors | Two triggers defined above the `CREATE TABLE` they attach to, so `CREATE TRIGGER ... ON public.views` failed | All four triggers moved after every `CREATE TABLE` |
   | 21 errors | `ALTER ... OWNER TO shiv` for a role that does not exist outside the original dev machine | Removed; a guarded `DO` block at the end reassigns ownership only when the role exists |
   | 4 errors | `citext` cannot be indexed on YugabyteDB (`INDEX on column of type 'user_defined_type' not yet supported`) | The two `UNIQUE` constraints are commented out and restored by `20261001160000_widen_username_to_33.sql` as `lower(...)` expression indexes; the two plain indexes are dropped |
   | 1 error | `tokens_id_seq` created twice - `tokens.id` is `bigserial`, and the dump also emitted an explicit `CREATE SEQUENCE` | Explicit creation removed |

   Applying the file to an empty database went from **28 errors to 0**, and all
   four migrations now run end to end from empty via `run_migrations.py`.

   A note on the `citext` change: `tags_name_key` and `users_email_key` are no
   longer created by this file. On vanilla PostgreSQL that would be a
   regression, so the statements are left in place commented out rather than
   deleted, and the follow-up migration rebuilds them as `lower(name)` /
   `lower(email)` unique indexes, which YugabyteDB does accept. Verified on a
   virgin database: `A@Example.com` and `a@example.COM` collide, as do `Rust`
   and `rust`.


