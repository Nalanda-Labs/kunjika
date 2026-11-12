-- Indexes for the remaining counts that scanned a whole table.
--
-- After this, every `count(*)` the backend runs can be answered from an
-- index alone. The counts that already could: all questions
-- (`posts_op_id_idx`), a question's answers (`posts_op_id_idx`), a user's
-- questions and answers (`posts_posted_by_idx`), a user's bookmarks
-- (`bookmarks_uid_idx`), tags and users (their primary keys), open reports
-- and unread notifications (their own partial indexes).
--
-- YugabyteDB notes (same constraints as the earlier migrations):
--   * DDL is NOT transactional, so every statement is idempotent
--     (IF NOT EXISTS) and the file can be re-run.
--   * `run_migrations.py` feeds this file to `psql`, which runs each
--     top-level statement in its own autocommit.
--   * A first index column without a direction is hash-partitioned and
--     cannot serve a range or an ordering, hence the explicit `DESC` below.

-- ---------------------------------------------------------------------------
-- Unanswered questions: `unanswered_questions_count`, and the unanswered
-- list, which reads the same rows ordered by `updated_at`. A partial index
-- holds exactly those rows, so the count reads all of it and nothing else,
-- and the list reads it in order. A question leaves the index when it gets
-- its first answer.
-- ---------------------------------------------------------------------------
CREATE INDEX IF NOT EXISTS posts_unanswered_idx
    ON public.posts (updated_at DESC)
    WHERE op_id = 0 AND answer_count = 0;

-- ---------------------------------------------------------------------------
-- Pinned questions: `pinned_count`, the pin limit check and the pinned rows
-- at the top of the front page. At most a handful of rows are ever pinned,
-- so this index stays tiny, where the count used to scan `posts`.
-- ---------------------------------------------------------------------------
CREATE INDEX IF NOT EXISTS posts_pinned_idx
    ON public.posts (id)
    WHERE pinned = true;

-- ---------------------------------------------------------------------------
-- Page views: "has this user (or this address) already viewed this question
-- in the last 15 minutes?" is counted on every question page load. Both
-- forms of the count give `qid`, so one index serves both: the signed-in
-- form matches on `userid`, the anonymous form filters on `ipaddress`.
-- ---------------------------------------------------------------------------
CREATE INDEX IF NOT EXISTS views_qid_viewer_idx
    ON public.views (qid, userid, ipaddress);
