# kunjika

## What is kunjika?
Kunjika is a minimal QA/forum application built using Rust, ntex, Svelte, YugabyeDB(changed
from CockroachDB because SQLx 0.9 does not work well with Cockroachdb ),
and Redis. Demo at https://kunjika.ashtavakra.org.

## Is it usable?
It is under active development and is being used in production. Look at demo site

## Is there a binary release for installation?
There is no binary release nor there is any plan to do so.
The installation must be done using source code. Perhaps later I would release a docker
config.

## Installation

Pick one of the two paths below. Both end with the same running app:

- frontend at http://localhost:3000
- backend API at http://localhost:8000/api/v1/
- mail UI (dev mails) at http://localhost:8026

### Option A: entire app with podman (recommended)

Prerequisites: `podman` and `podman-compose` (or `docker` + `docker compose` —
the file works with both).

```bash
# from the repo root; first boot is slow (YugabyteDB ~30-60s + Rust build)
podman-compose -f podman-compose.yaml up --build
# or detached:
podman-compose -f podman-compose.yaml up --build -d
```

What that does, in order: starts YugabyteDB, Redis and Mailpit; the one-shot
`migrate` service waits for YSQL, creates the `kunjika` database if missing,
and applies `backend/migrations/*.sql` (tracked in `kunjika_migrations`);
only then do `backend` (Rust/ntex, port 8000) and `frontend` (SvelteKit node
build, port 3000) start. So "bring up containers and the entire app after
running migrations" is exactly what `up` does.

```bash
podman-compose -f podman-compose.yaml ps     # status
podman-compose -f podman-compose.yaml logs -f backend   # follow API logs
podman-compose -f podman-compose.yaml down              # stop, keep DB data
podman-compose -f podman-compose.yaml down -v           # stop, drop DB data
```

Notes:

- The database must be YugabyteDB, not vanilla PostgreSQL: migrations use
  `USING lsm` indexes and `citext` (see `TESTING.md`).
- The backend container builds from the bind-mounted `./backend` sources, so
  local edits are picked up on restart; the `backend-target` volume caches
  the Cargo build. SMTP goes to the `mailpit` container via the new
  `KUNJIKA_MAIL_HOST`/`KUNJIKA_MAIL_PORT` env overrides in
  `backend/src/config.rs`.
- Config inside compose comes from `backend/template-sample.json` (copied to
  `template.compose.json` on first start) plus the `KUNJIKA_*` environment in
  `podman-compose.yaml`. To customise mail sender, secrets or JWT keys, edit
  `template.compose.json` or mount your own file.

### Option B: entire app without podman (host services)

Prerequisites: Rust toolchain (`cargo`), Node 22 + pnpm/npm, Python 3,
YugabyteDB (YSQL on 5433), Redis (6379), and a plaintext SMTP relay on
127.0.0.1:25 (e.g. a local Mailpit/Postfix; `config.rs` honours
`Config::mail_host`/`mail_port`, default 127.0.0.1:25).

1. Start YugabyteDB + Redis + Mailpit (see `TESTING.md` section 1 for the
   exact `podman run` commands and ports; install the same daemons natively
   if you want zero containers).
2. Create the database and run migrations (one of):

```bash
cd backend/migrations && ./run_migrations.sh --status && ./run_migrations.sh
# or: python3 run_migrations.py  (needs pip install "psycopg[binary]")
```

   Target is `$DATABASE_URL`, falling back to `backend/.env`:
   `DATABASE_URL=postgres://yugabyte:yugabyte@127.0.0.1:5433/kunjika`.
   Use `--baseline` (not a re-run) when adopting a DB whose schema is
   already current. Restart the backend after DDL changes (sqlx caches
   prepared statements — see `TESTING.md`).
3. Configure the backend: copy `backend/template-sample.json` to
   `backend/template.json` and set `sql` (e.g.
   `postgres://yugabyte:yugabyte@127.0.0.1:5433/kunjika`), `redis`
   (`redis://127.0.0.1:6379`), `mail_host`/`mail_port`, `listen`
   (`127.0.0.1:8000`), `host` (frontend origin), JWT keys, etc.
4. Build and run the backend:

```bash
cd backend && cargo build && ./target/debug/kunjika -c template.json -v 1
# dev loop: cargo watch -x 'run -- -v 2 -c template.json'
```

   The `listen` field from the config is now honoured (previously the bind
   was hardcoded to `127.0.0.1:8000`); compose sets it to `0.0.0.0:8000`.
5. Install and run the frontend (`frontend/src/env.js` points at
   `http://localhost:8000/api/v1`):

```bash
cd frontend && npm i -g pnpm && pnpm i && pnpm dev     # dev on :5173
# production: pnpm build && node build               # serves on :3000
```

   Whitelist the frontend origin in the backend CORS list
   (`backend/src/main.rs`: 5173 dev, 3000 prod) if you serve it elsewhere.
   For per-directory detail see `backend/README.md` / `frontend/README.md`.

## Running the test suites

Full details, container setup and env knobs live in [`TESTING.md`](TESTING.md);
this is the short version. All three suites need the infra from section 1 of
that file (YugabyteDB with a migrated `kunjika_test` database, Redis,
plaintext Mailpit on 2525/8026) plus a backend built from `backend/`.

| Suite | Command | Result |
| --- | --- | --- |
| Backend REST (`cargo test`) | `cd backend/test-harness && cargo test` | 33 passed |
| Backend unit (`cargo test --bin kunjika`) | `cd backend && cargo test --bin kunjika` | 3 passed |
| Frontend e2e (Playwright) | `cd frontend && npx playwright test -c playwright.e2e.config.ts` | 13 passed |

Order matters: start infra, apply migrations, then run the suites.

```bash
# 1. infra (YSQL :5433, Redis :6379, Mailpit :2525/:8026) — TESTING.md section 1
# 2. migrations
cd backend/migrations && ./run_migrations.sh --status && ./run_migrations.sh
# 3a. backend REST suite (spawns the backend itself via an LD_PRELOAD shim
#     that redirects its SMTP :25 to Mailpit :2525)
cd backend/test-harness && cargo test
# 3b. backend unit tests (need DATABASE_URL / backend/.env pointing at YSQL)
cd backend && cargo test --bin kunjika
# 3c. frontend e2e (needs backend on 127.0.0.1:8000; starts preview on :5173)
cd frontend && npx playwright install chromium \
  && npx playwright test -c playwright.e2e.config.ts
```

`backend/test-harness` is a separate crate driving the running backend over
HTTP; `KUNJIKA_TEST_NO_SPAWN=1 cargo test -- --test-threads=1` runs it
against an already-running backend instead. `sqlx::query!` macros are
compile-time checked, so building/testing the backend also needs a live DB
via `backend/.env` (`DATABASE_URL=...`). If JWT login answers 502
`InvalidKeyFormat`, regenerate the throwaway test keys with
`./backend/test-harness/scripts/gen_test_keys.sh`.



## Why another QA?
It is not about QA/forum only. Most web software is written using inefficient languages
like PHP, Python, Java, Ruby and so on. It is an attempt to develop a web application
in Rust which is very efficient and resource-friendly.

Discourse is very nice forum software but I find it bloated with features but for most
use cases it should work nicely. If you want a lean/mean QA/forum only then you should
choose Kunjika. The other reason is the license. Kunjika is true free software under
GNU GPL and not under some other open source license.

There are some software projects being done in Golang for efficiency. However, I think
that the philosophy behind development of Golang is horribly wrong. Rust is the
silver bullet of programming.

If you want more features you can try Apache Answer written in Golang.

*Kunjika will not integrate with any closed-source/non-GPL applications. Please do not create
issues for such integrations.*
