-- Full-text search over questions only.
--
-- In this schema a question is a row in `posts` whose `title` is neither NULL
-- nor empty (answers carry the body in `description` and leave `title`
-- NULL or empty). `description` is NOT NULL but may itself be empty in
-- degenerate rows.
--
-- The partial predicate below therefore indexes a row exactly when it is a
-- question:
--
--     title IS NOT NULL AND title <> ''
--
-- deliberately ignoring `description`, so:
--   * answers (NULL or '' title) are never indexed, keeping the index small
--     and answer text out of question search results;
--   * malformed rows with an empty title are excluded too.
--
-- `coalesce(title, '') || ' ' || description` feeds `to_tsvector`, and the
-- GIN index makes `@@ plainto_tsquery(...)` + `ts_rank_cd(...)` ranking fast.
-- The statement is idempotent (`IF NOT EXISTS`), matching the convention of
-- the other migrations in this directory.
CREATE INDEX IF NOT EXISTS posts_search_idx
    ON public.posts
    USING gin (to_tsvector('english', coalesce(title, '') || ' ' || description))
    WHERE title IS NOT NULL AND title <> '';
