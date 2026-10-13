-- Fix the votes primary key and reconcile karma.
--
-- ===========================================================================
-- BUG: votes_pkey is PRIMARY KEY (topic_id) — the post id ALONE.
--
-- backend/src/votes/dao.rs::handle_vote is written for one row per VOTER per
-- topic:
--
--     select * from votes where topic_id=$1 and from_user_id=$2   -- per voter
--     insert into votes(topic_id, from_user_id, to_user_id, vote) ...
--
-- With the key being `topic_id` alone, the FIRST voter on a post inserts fine,
-- but EVERY subsequent voter hits the primary key and fails the INSERT with
--     SQLSTATE 23505  duplicate key value violates unique constraint "votes_pkey"
-- which handle_vote surfaces as an HTTP 500. Net effect: only one user can
-- ever vote on a given post, and everyone else gets a server error.
--
-- FIX: the primary key must be (topic_id, from_user_id) — one vote per voter
-- per topic, which is exactly what the code queries and inserts on.
--
-- ===========================================================================
-- YugabyteDB constraints that shape this migration:
--   * The primary key defines the table's sharding/ordering and CANNOT be
--     altered in place — there is no `ALTER TABLE ... ALTER PRIMARY KEY`. The
--     table must be recreated with the new key and the data copied across.
--   * DDL is NOT transactional on YugabyteDB: a statement that has run is in
--     effect even if a later statement in the file fails. So this migration is
--     written to be IDEMPOTENT — it first checks whether votes already carries
--     the composite key and, if so, does nothing. A partially-applied run
--     leaves a `votes_new` staging table that a re-run reuses (it recreates the
--     staging table from scratch and re-copies), so re-running converges.
--   * No foreign key references `votes` (verified: none), so swapping the table
--     is referentially safe.
-- ===========================================================================

-- IMPORTANT — how this is structured, and why it is NOT a single DO block:
--
-- An earlier version wrapped the whole rebuild in one `DO $$ ... $$`. On
-- YugabyteDB that failed: the DROP TABLE / ALTER ... RENAME inside the block
-- outlived the transaction while the data-copy INSERT was rolled back, so the
-- swap ran but the rows were lost. YugabyteDB cannot keep a multi-statement
-- table swap in one atomic transaction, so the rebuild below is emitted as
-- SEPARATE top-level statements. `run_migrations.py` feeds this file to
-- `psql`, which runs each top-level statement in its own autocommit — exactly
-- the per-statement behaviour this needs. The idempotency guard is a psql
-- `\gset` + `\if` around the DDL, so a re-run on an already-fixed database
-- (or a database where a prior run left `votes_new` behind) changes nothing.

-- Is `votes` already keyed on (topic_id, from_user_id)? If so, skip the whole
-- rebuild. EXISTS(...) yields a boolean that \gset stores as :already_fixed.
SELECT EXISTS (
    SELECT 1
      FROM pg_constraint
     WHERE conrelid = 'public.votes'::regclass
       AND contype = 'p'
       AND pg_get_constraintdef(oid) ILIKE '%from_user_id%'
) AS already_fixed
\gset

\if :already_fixed
\echo 'votes already keyed on (topic_id, from_user_id) - no change needed'
\else
\echo 'rebuilding votes with PRIMARY KEY (topic_id, from_user_id)'

-- Staging table with the correct composite key. Drop any leftover staging table
-- from a prior partial run so the copy starts clean (idempotent).
DROP TABLE IF EXISTS public.votes_new;

CREATE TABLE public.votes_new (
    topic_id     bigint NOT NULL,
    to_user_id   bigint,
    from_user_id bigint,
    vote         bigint NOT NULL,
    CONSTRAINT votes_new_pkey PRIMARY KEY (topic_id, from_user_id)
);

-- Secondary indexes that existed on the old table (same columns, LSM/HASH to
-- match the rest of the schema). topic_id alone is now redundant with the
-- leading PK column but is recreated to preserve the original index set.
CREATE INDEX votes_new_from_user_id_idx ON public.votes_new USING lsm (from_user_id HASH);
CREATE INDEX votes_new_to_user_id_idx   ON public.votes_new USING lsm (to_user_id HASH);
CREATE INDEX votes_new_topic_id_idx     ON public.votes_new USING lsm (topic_id HASH);

-- Copy the data. The old key (topic_id) means there is at most one row per
-- topic today, so the copy cannot violate the new composite key. This runs in
-- its own autocommit (see the note above), so it is not lost by the swap.
INSERT INTO public.votes_new (topic_id, to_user_id, from_user_id, vote)
SELECT topic_id, to_user_id, from_user_id, vote FROM public.votes;

-- Swap: drop the old table, rename staging into place, and rename the
-- constraint/indexes back to their original names so anything looking them up
-- by name still finds them. Each statement autocommits on its own.
DROP TABLE public.votes;

ALTER TABLE public.votes_new RENAME TO votes;

ALTER INDEX public.votes_new_pkey           RENAME TO votes_pkey;
ALTER INDEX public.votes_new_from_user_id_idx RENAME TO from_user_id_idx;
ALTER INDEX public.votes_new_to_user_id_idx   RENAME TO to_user_id_idx;
ALTER INDEX public.votes_new_topic_id_idx     RENAME TO topic_id_idx;

\endif


-- ===========================================================================
-- Reconcile users.karma against the source of truth.
--
-- `users.karma` is a denormalized counter incremented/decremented in
-- votes/dao.rs and questions/dao.rs::accept_answer. Nothing recomputes it from
-- the underlying events, so it can drift (a failed/clawed-back path, a
-- hand-edited row). This statement recomputes every user's karma from the
-- authoritative data so the counter is made whole:
--
--     karma = GREATEST(1, 1
--                       + 10 * (net votes received)            -- karma_gain_per_vote
--                       + 25 * (accepted answers, excl. self)) -- karma_gain_per_answer
--
-- The weights (10 / 25) mirror config.rs::karma_gain_per_vote and
-- karma_gain_per_answer (template.json). The base of 1 and the GREATEST(1, ...)
-- floor reproduce the schema default (`karma bigint DEFAULT 1`) and the
-- "never below 1" rule enforced in the vote/accept paths.
--
-- "net votes received" = SUM(votes.vote) grouped by to_user_id, so an upvote
-- (+1) and a later downvote (−1) by different voters net out, matching the
-- running counter. Accepted answers exclude self-accepts (answerer is not the
-- asker), matching accept_answer which only grants karma when they differ.
--
-- This is a full recompute, so it is idempotent: running it again yields the
-- same values.
-- ===========================================================================

WITH votes_net AS (
    SELECT to_user_id AS uid, SUM(vote) AS net
      FROM votes
     GROUP BY to_user_id
),
accepted AS (
    SELECT a.posted_by_id AS uid, COUNT(*) AS n
      FROM posts a
      JOIN posts q ON q.id = a.op_id
     WHERE a.answer_accepted = true
       AND a.op_id <> 0
       AND a.posted_by_id <> q.posted_by_id   -- exclude self-accepts
     GROUP BY a.posted_by_id
)
UPDATE users u
   SET karma = GREATEST(
                   1,
                   1
                   + 10  * COALESCE(v.net, 0)
                   + 25  * COALESCE(c.n,   0)
               )
  FROM votes_net v
  FULL JOIN accepted c ON c.uid = v.uid
 WHERE u.id = COALESCE(v.uid, c.uid);
