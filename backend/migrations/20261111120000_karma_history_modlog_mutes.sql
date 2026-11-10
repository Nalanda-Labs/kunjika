-- Karma history, the moderation log, muted threads, and a search index over
-- answers.
--
-- YugabyteDB notes (same constraints as the earlier migrations):
--   * DDL is NOT transactional, so every statement is idempotent
--     (IF NOT EXISTS / guarded inserts) and the file can be re-run.
--   * `run_migrations.py` feeds this file to `psql`, which runs each
--     top-level statement in its own autocommit.

-- ---------------------------------------------------------------------------
-- Karma history: one row per change to a user's karma.
--
--   delta     the change (positive or negative)
--   reason    vote_up | vote_down | accepted | unaccepted | post_deleted |
--             opening_balance
--   post_id   the post the change is about; NULL for the opening balance.
--             Kept after the post is deleted, so the history stays complete.
--   actor_id  who caused it (the voter, the asker, whoever deleted the
--             post). Recorded for audit; the API never returns it, so votes
--             stay as private as they were.
--
-- `users.karma` remains the number that is displayed. It is never allowed
-- below 1, so it can differ from 1 + sum(delta) for a user who was voted
-- down past that floor.
-- ---------------------------------------------------------------------------
CREATE SEQUENCE IF NOT EXISTS public.karma_events_id_seq;

CREATE TABLE IF NOT EXISTS public.karma_events (
    id bigint NOT NULL DEFAULT nextval('public.karma_events_id_seq'::regclass),
    user_id bigint NOT NULL,
    delta bigint NOT NULL,
    reason character varying(24) NOT NULL,
    post_id bigint,
    actor_id bigint,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT karma_events_pkey PRIMARY KEY (id),
    CONSTRAINT karma_events_reason_check CHECK (
        reason IN ('vote_up', 'vote_down', 'accepted', 'unaccepted', 'post_deleted',
                   'opening_balance')
    )
);

-- The history is read newest-first per user with a (created_at, id) cursor.
CREATE INDEX IF NOT EXISTS karma_events_user_created_idx
    ON public.karma_events (user_id, created_at DESC, id DESC);

CREATE INDEX IF NOT EXISTS karma_events_post_idx ON public.karma_events (post_id);

-- Karma earned before this table existed cannot be itemised: the votes table
-- holds only each voter's current vote, not what happened when. Each user
-- with karma above the starting value of 1 gets a single opening entry for
-- the difference, so their history adds up to what their profile shows.
-- Guarded so a re-run does not add the entries twice.
INSERT INTO public.karma_events (user_id, delta, reason)
SELECT u.id, u.karma - 1, 'opening_balance'
FROM public.users u
WHERE u.karma > 1
  AND NOT EXISTS (SELECT 1 FROM public.karma_events);

-- ---------------------------------------------------------------------------
-- Moderation log: who deleted, closed, reopened, suspended or reinstated
-- what. `details` keeps a description of the target (a deleted post's title,
-- for instance), since the row it refers to may no longer exist.
-- ---------------------------------------------------------------------------
CREATE SEQUENCE IF NOT EXISTS public.moderation_log_id_seq;

CREATE TABLE IF NOT EXISTS public.moderation_log (
    id bigint NOT NULL DEFAULT nextval('public.moderation_log_id_seq'::regclass),
    actor_id bigint NOT NULL,
    action character varying(24) NOT NULL,
    post_id bigint,
    target_user_id bigint,
    details character varying(512) DEFAULT '' NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT moderation_log_pkey PRIMARY KEY (id),
    CONSTRAINT moderation_log_action_check CHECK (
        action IN ('delete_post', 'close_question', 'reopen_question',
                   'suspend_user', 'reinstate_user')
    )
);

CREATE INDEX IF NOT EXISTS moderation_log_created_idx
    ON public.moderation_log (created_at DESC, id DESC);

-- ---------------------------------------------------------------------------
-- Muted threads: a user who mutes a question gets no notifications or
-- emails about new answers in it.
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS public.thread_mutes (
    user_id bigint NOT NULL,
    question_id bigint NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT thread_mutes_pkey PRIMARY KEY (user_id, question_id)
);

CREATE INDEX IF NOT EXISTS thread_mutes_question_idx ON public.thread_mutes (question_id);

-- ---------------------------------------------------------------------------
-- Full-text search over answers, so a search also finds questions whose
-- answers match. The counterpart of `posts_search_idx` (questions only, see
-- 20261009120000_posts_search_idx.sql): an answer is a post with a parent.
-- ---------------------------------------------------------------------------
CREATE INDEX IF NOT EXISTS posts_answers_search_idx
    ON public.posts
    USING gin (to_tsvector('english', description))
    WHERE op_id <> 0;
