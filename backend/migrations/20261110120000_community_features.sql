-- Community features: closing questions, suspending users, edit history,
-- in-app notifications, an email opt-out, and following tags.
--
-- YugabyteDB notes (same constraints as the earlier migrations):
--   * DDL is NOT transactional, so every statement is idempotent
--     (IF NOT EXISTS) and the file can be re-run after a partial failure.
--   * `run_migrations.py` feeds this file to `psql`, which runs each
--     top-level statement in its own autocommit.
--   * Restart the backend afterwards: sqlx caches prepared statements and
--     the ALTER TABLEs below change the shape of `posts` and `users`.

-- ---------------------------------------------------------------------------
-- Closing a question (optionally as a duplicate of another one).
--
-- A closed question stays readable but takes no new answers. `closed_at` is
-- the flag: NULL means open. `duplicate_of_id` points at the original when
-- the reason for closing is "already asked"; it is NULL for a plain close.
-- No foreign key: the original may be deleted later, and the delete path
-- clears the pointer itself.
-- ---------------------------------------------------------------------------
ALTER TABLE public.posts ADD COLUMN IF NOT EXISTS closed_at timestamp with time zone;
ALTER TABLE public.posts ADD COLUMN IF NOT EXISTS closed_by_id bigint;
ALTER TABLE public.posts ADD COLUMN IF NOT EXISTS duplicate_of_id bigint;

-- ---------------------------------------------------------------------------
-- Email opt-out for "new answer" mails. Suspension needs no new column:
-- `users.status` ('normal' | 'suspended') has existed since the first
-- migration, it just never had a writer.
-- ---------------------------------------------------------------------------
ALTER TABLE public.users
    ADD COLUMN IF NOT EXISTS email_notifications boolean DEFAULT true NOT NULL;

-- ---------------------------------------------------------------------------
-- Edit history. One row per edit, holding the post as it was BEFORE that
-- edit; the current text stays in `posts`. `tags` is the comma-joined tag
-- list of a question at that time (NULL for answers).
-- ---------------------------------------------------------------------------
CREATE SEQUENCE IF NOT EXISTS public.post_revisions_id_seq;

CREATE TABLE IF NOT EXISTS public.post_revisions (
    id bigint NOT NULL DEFAULT nextval('public.post_revisions_id_seq'::regclass),
    post_id bigint NOT NULL,
    title character varying(256),
    description character varying(1000000) NOT NULL,
    tags text,
    edited_by_id bigint,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT post_revisions_pkey PRIMARY KEY (id)
);

CREATE INDEX IF NOT EXISTS post_revisions_post_idx
    ON public.post_revisions (post_id, created_at DESC);

-- ---------------------------------------------------------------------------
-- In-app notifications.
--
--   kind          what happened: answer | reply | accepted | upvote |
--                 tag_question
--   user_id       who is being told
--   actor_id      who did it
--   question_id   the question to link to
--   post_id       the post it is about (the question itself, or an answer)
--   read_at       NULL while unread
-- ---------------------------------------------------------------------------
CREATE SEQUENCE IF NOT EXISTS public.notifications_id_seq;

CREATE TABLE IF NOT EXISTS public.notifications (
    id bigint NOT NULL DEFAULT nextval('public.notifications_id_seq'::regclass),
    user_id bigint NOT NULL,
    kind character varying(24) NOT NULL,
    actor_id bigint,
    question_id bigint,
    post_id bigint,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    read_at timestamp with time zone,
    CONSTRAINT notifications_pkey PRIMARY KEY (id),
    CONSTRAINT notifications_kind_check CHECK (
        kind IN ('answer', 'reply', 'accepted', 'upvote', 'tag_question')
    )
);

-- The list is read newest-first per user with a (created_at, id) cursor.
CREATE INDEX IF NOT EXISTS notifications_user_created_idx
    ON public.notifications (user_id, created_at DESC, id DESC);

-- The unread badge counts per user.
CREATE INDEX IF NOT EXISTS notifications_unread_idx
    ON public.notifications (user_id)
    WHERE read_at IS NULL;

CREATE INDEX IF NOT EXISTS notifications_post_idx
    ON public.notifications (post_id);

-- ---------------------------------------------------------------------------
-- Followed tags: a new question in a followed tag notifies the follower.
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS public.tag_follows (
    user_id bigint NOT NULL,
    tag_id bigint NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT tag_follows_pkey PRIMARY KEY (user_id, tag_id)
);

CREATE INDEX IF NOT EXISTS tag_follows_tag_idx ON public.tag_follows (tag_id);
