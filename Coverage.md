# Test coverage

Measured 2026-11-07 on the working tree (uncommitted changes on top of `94cb462`).

## Summary

| Measure | Result |
|---|---|
| Backend lines | **83.24%** (1753 of 2106) |
| Backend functions | **93.70%** (223 of 238) |
| Backend regions | **81.99%** (2827 of 3448) |
| API routes reached by a test | **47 of 47** |
| Frontend pages opened by a test | **19 of 19** (`/logout` is exercised through its form action) |
| Tests run | 67 e2e (Playwright), 41 integration (Rust harness), 5 unit |

Branch coverage is not reported: it needs a nightly compiler (`-Z coverage-options=branch`).
Frontend JavaScript line coverage is not measured; the page and route counts above stand in for it.

## How it was measured

The backend was built with `-C instrument-coverage` (LLVM 23, matching rustc 1.99).
One instrumented server ran while the unit tests, the full Playwright suite and the
full Rust integration suite (with `KUNJIKA_TEST_NO_SPAWN=1`) ran against it. The
server was then stopped with SIGTERM so it wrote its profile, and all profiles were
merged. Only files under `backend/src/` are counted.

## Per file

| File | Lines | Missed | Line cover | Functions | Function cover |
|---|---:|---:|---:|---:|---:|
| `main.rs` | 26 | 0 | 100.00% | 3 | 100.00% |
| `config.rs` | 150 | 46 | 69.33% | 10 | 50.00% |
| `middlewares/auth.rs` | 97 | 33 | 65.98% | 5 | 100.00% |
| `questions/routes.rs` | 622 | 91 | 85.37% | 54 | 98.15% |
| `questions/dao.rs` ¹ | 69 | 0 | 100.00% | 36 | 100.00% |
| `questions/question.rs` | 15 | 0 | 100.00% | 2 | 100.00% |
| `users/routes.rs` | 656 | 106 | 83.84% | 38 | 100.00% |
| `users/dao.rs` ¹ | 49 | 14 | 71.43% | 22 | 68.18% |
| `users/token.rs` | 51 | 2 | 96.08% | 2 | 100.00% |
| `users/user.rs` | 23 | 0 | 100.00% | 5 | 100.00% |
| `tags/routes.rs` | 87 | 15 | 82.76% | 11 | 100.00% |
| `tags/dao.rs` ¹ | 11 | 0 | 100.00% | 5 | 100.00% |
| `votes/routes.rs` | 29 | 8 | 72.41% | 3 | 100.00% |
| `votes/dao.rs` ¹ | 2 | 0 | 100.00% | 1 | 100.00% |
| `uploads/routes.rs` | 42 | 4 | 90.48% | 10 | 100.00% |
| `uploads/save.rs` | 68 | 16 | 76.47% | 9 | 100.00% |
| `utils/request.rs` | 45 | 17 | 62.22% | 9 | 77.78% |
| `utils/security.rs` | 38 | 0 | 100.00% | 9 | 100.00% |
| `utils/slug.rs` | 19 | 1 | 94.74% | 2 | 100.00% |
| `utils/verify_user.rs` | 7 | 0 | 100.00% | 2 | 100.00% |
| **Total** | **2106** | **353** | **83.24%** | **238** | **93.70%** |

¹ **DAO line counts are not real line coverage.** Every DAO method sits inside
`#[async_trait]`, which rewrites the body into a macro-generated async block that
LLVM's coverage mapping does not attribute to source lines. For the `*/dao.rs` files
only "was this method called" is measured. Every live DAO method is called; their
individual lines and branches inside are not counted.

## What is not tested

Every route handler runs in at least one test. What remains is mostly error paths:

- **Infrastructure failures.** Database, Redis and SMTP errors (`db_error`,
  `internal_error`, the `Err(_)` arms in login, refresh and logout) are never
  triggered, because the test services are always up.
- **Malformed sessions.** Most of `middlewares/auth.rs`'s rejection branches (missing
  XSRF header, missing or malformed cookie, user deleted mid-session) are only
  partly hit.
- **Upload failures.** In `uploads/save.rs`: a client aborting mid-upload, an empty
  multipart body, and a failed disk write.
- **Startup and config.** In `config.rs`: the `KUNJIKA_*` environment overrides, the
  panics on unreachable services, and `Config::show`.
- **Pagination.** `direction: "back"` paging on the question, tag and user lists.
- **Defaults.** For example, the report queue's default `status` filter is never used,
  because the frontend and the tests always send one.

## Dead code

- `IUser::update_username`, `update_title`, `update_name`, `update_designation`,
  `update_location`, `get_links` and `update_links` (in `users/dao.rs`) are never
  called. Their routes are commented out in `users/routes.rs`, and
  `POST /{id}/update-profile` replaced them. They make up all of that file's missed
  functions.
- `Config::show` (in `config.rs`) is never called.

## Bugs found while measuring (now fixed)

Reading the code that never ran in the first measurement found these. Each fix has a
test.

1. **Any logged-in user could pin or unpin any question** to the top of the front
   page. Pinning is now for moderators and superusers only (403 otherwise). The Pin
   control is hidden from everyone else.
2. **Pinning a question that did not exist crashed the request handler** (`unwrap()`
   on a missing row). It now returns 404, as does trying to pin an answer.
3. **Once 5 questions were pinned, none could be unpinned,** and the handler still
   reported success. Unpinning now always works; a sixth pin gets 409 with a message.
4. **Tag info's 100-character minimum was declared but never enforced.** It now
   returns 400.
5. **Token rotation could log users out mid-request.** A page load refreshes the
   session; requests still in flight with the previous token were rejected with 401.
   Rotated tokens now stay valid for 10 seconds (`ROTATION_GRACE_SECS`) before they
   expire. Logout still revokes immediately.

New tests: `backend/test-harness/tests/api_moderation.rs` (pinning, the report queue,
editing tag info, listing your own bookmarks), plus e2e checks that regular users see
no Pin control and get 403 from the API.

## Regenerating this report

Requires the test services from `TESTING.md` (YugabyteDB, Redis, STARTTLS Mailpit)
and `llvm-profdata` and `llvm-cov` with the same LLVM major version as `rustc -vV`.

```bash
cd backend
PROF=/tmp/kunjika-cov && rm -rf $PROF && mkdir -p $PROF
export CARGO_TARGET_DIR=target/coverage RUSTFLAGS="-C instrument-coverage"
# Build scripts and proc macros are instrumented too; without this they drop
# default_*.profraw files into backend/.
export LLVM_PROFILE_FILE="$PROF/build-%p-%m.profraw"

cargo build
LLVM_PROFILE_FILE="$PROF/unit-%p-%m.profraw" cargo test --bin kunjika

LLVM_PROFILE_FILE="$PROF/server-%p-%m.profraw" \
  ./target/coverage/debug/kunjika -c template.test.json -v 1 &
(cd ../frontend && npx playwright test -c playwright.e2e.config.ts)
(cd test-harness && unset CARGO_TARGET_DIR RUSTFLAGS && \
  KUNJIKA_TEST_NO_SPAWN=1 KUNJIKA_TEST_MAILPIT_URL=http://127.0.0.1:8026 \
  cargo test -- --test-threads=1)
kill -TERM %1 && wait   # a clean exit writes the server profile

llvm-profdata merge -sparse $PROF/*.profraw -o $PROF/kunjika.profdata
UNIT=$(ls -t target/coverage/debug/deps/kunjika-* | grep -v '\.d$' | head -1)
llvm-cov report target/coverage/debug/kunjika -object "$UNIT" \
  -instr-profile=$PROF/kunjika.profdata \
  -ignore-filename-regex='(\.cargo/registry|/rustc/|\.rustup/|/target/)'
# Browsable HTML: replace `report` with `show -format=html -output-dir=$PROF/html`
```
