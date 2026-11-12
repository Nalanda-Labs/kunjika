-- Drop the views the backend no longer reads, and add the indexes that the
-- queries replacing them need.
--
-- YugabyteDB notes (same constraints as the earlier migrations):
--   * DDL is NOT transactional, so every statement is idempotent
--     (IF EXISTS / IF NOT EXISTS) and the file can be re-run.
--   * `run_migrations.py` feeds this file to `psql`, which runs each
--     top-level statement in its own autocommit.
--   * Deploy the backend that comes with this migration: older builds still
--     query these views.

-- ---------------------------------------------------------------------------
-- Per-user counts.
--
-- `answers_count`, `questions_count_by_user` and `bookmarks_count` were
-- GROUP BY views, so a user with nothing to count had no row in them, and the
-- profile's questions, answers and bookmarks tabs answered 500 for such a
-- user. The backend now counts from `posts` and `bookmarks` directly, which
-- always returns a row.
-- ---------------------------------------------------------------------------
DROP VIEW IF EXISTS public.answers_count;
DROP VIEW IF EXISTS public.questions_count_by_user;
DROP VIEW IF EXISTS public.bookmarks_count;

-- A user's questions (op_id = 0) and answers (op_id <> 0) are counted, and
-- listed, by author. There was no index on the author, so both scanned
-- `posts`. `op_id` is in the index so that a count needs the index alone.
CREATE INDEX IF NOT EXISTS posts_posted_by_idx
    ON public.posts (posted_by_id, op_id);

-- A user's bookmarks. `bookmarks_ids` leads with `qid`, so it cannot serve a
-- lookup by user.
CREATE INDEX IF NOT EXISTS bookmarks_uid_idx
    ON public.bookmarks (uid, created_at DESC);

-- ---------------------------------------------------------------------------
-- Tag popularity.
--
-- `daily_tags_by_popularity` and `weekly_tags_by_popularity` were joined into
-- every tag list by tag NAME, each aggregating `post_tags` separately (and
-- sorting the result, which a join discards). The backend now reads both
-- counts from one aggregate over the last 7 days, joined by tag id.
--
-- "Daily" also changes meaning slightly: it was "since midnight yesterday"
-- (24 to 48 hours), and is now the last 24 hours. Weekly is the last 7 days.
-- ---------------------------------------------------------------------------
DROP VIEW IF EXISTS public.daily_tags_by_popularity;
DROP VIEW IF EXISTS public.weekly_tags_by_popularity;

-- The aggregate reads the last week of `post_tags`. `DESC` makes this a
-- range index (a first column without a direction is hash-partitioned in
-- YugabyteDB and cannot serve a range), and `tag_id` is included so that
-- the aggregate needs the index alone.
CREATE INDEX IF NOT EXISTS post_tags_created_idx
    ON public.post_tags (created_at DESC) INCLUDE (tag_id);

-- ---------------------------------------------------------------------------
-- `active_tokens` selected tokens created more than 15 minutes in the FUTURE
-- (`NOW() + interval`, where minus was meant), so it was always empty, and
-- nothing ever read it. Expired tokens are purged by the `tokens` triggers.
-- ---------------------------------------------------------------------------
DROP VIEW IF EXISTS public.active_tokens;
