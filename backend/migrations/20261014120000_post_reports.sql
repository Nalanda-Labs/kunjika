-- Post reports (moderation queue).
--
-- The frontend has always rendered "Report" links on questions and answers,
-- but no backend ever existed for them: there was no report table and no
-- report endpoint, so the links were dead (they pointed at a nonexistent
-- `/report/{id}` page). This migration creates the table behind the new
-- reporting flow:
--
--   * any logged-in user can report a post (question or answer) for one of
--     a fixed set of reasons, at most 5 reports per day (rate limit);
--   * a user cannot report their own post and cannot file a second OPEN
--     report on the same post;
--   * moderators and superusers list reports (`POST /reports/`) and review
--     them (`POST /reports/{id}/review/`); the "actioned" outcome deletes
--     the reported post.
--
-- YugabyteDB notes (same constraints as the earlier migrations):
--   * DDL is NOT transactional, so every statement is idempotent
--     (IF NOT EXISTS / conditional inserts) and the file can be re-run.
--   * `run_migrations.py` feeds this file to `psql`, which runs each
--     top-level statement in its own autocommit.
--   * BIGSERIAL requires the extension-created backing sequence pattern
--     used by the initial migration's `*_id_seq` sequences.

CREATE TABLE IF NOT EXISTS public.post_reports (
    id bigint NOT NULL DEFAULT nextval('public.posts_id_seq'::regclass),
    post_id bigint NOT NULL,
    reported_by_id bigint NOT NULL,
    reason character varying(32) NOT NULL,
    details character varying(2000) DEFAULT '' NOT NULL,
    status character varying(16) DEFAULT 'open' NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    reviewed_by_id bigint,
    reviewed_at timestamp with time zone,
    CONSTRAINT post_reports_pkey PRIMARY KEY (id),
    CONSTRAINT post_reports_reason_check CHECK (
        reason IN ('spam', 'abusive', 'off-topic', 'low-quality', 'duplicate', 'other')
    ),
    CONSTRAINT post_reports_status_check CHECK (
        status IN ('open', 'reviewed', 'dismissed', 'actioned')
    )
);

-- One OPEN report per user per post. Re-reporting after a dismissal/action
-- is allowed, which is why the uniqueness only covers the open state.
CREATE UNIQUE INDEX IF NOT EXISTS post_reports_open_unique
    ON public.post_reports (post_id, reported_by_id)
    WHERE status = 'open';

-- Moderation queue reads newest-open-first; daily rate-limit counts per user.
CREATE INDEX IF NOT EXISTS post_reports_status_created_idx
    ON public.post_reports (status, created_at DESC);

CREATE INDEX IF NOT EXISTS post_reports_reporter_created_idx
    ON public.post_reports (reported_by_id, created_at DESC);

CREATE INDEX IF NOT EXISTS post_reports_post_idx
    ON public.post_reports (post_id);
