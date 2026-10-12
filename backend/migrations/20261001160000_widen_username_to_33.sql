-- Widen users.username to match the API validator in
-- backend/src/users/user.rs:
--
--     #[validate(length(min = 3, max = 33), custom(function = "validate_username"))]
--     pub username: String,
--
-- The column in 20220620050241_initial.sql is `character varying(10)`, so an
-- 11..33 character username passed validation and then failed the INSERT with
--
--     SQLSTATE 22001 - value too long for type character varying(10)
--
-- which users::routes::register maps to a misleading
--
--     HTTP 409 {"status":"fail","message":"Either email or username is taken"}
--
-- i.e. a length/DB error was reported to the user as a name collision.
--
-- This is a widening change (the new type is a supertype of the old one), so
-- existing rows are untouched and the UNIQUE index users_name_key is rebuilt.

ALTER TABLE public.users
    ALTER COLUMN username TYPE character varying(33);

-- ---------------------------------------------------------------------------
-- Guard: keep `users.username` UNIQUE.
--
-- `20220620050241_initial.sql` already adds `users_name_key UNIQUE (username)`,
-- so on a correctly migrated database this block is a no-op. It is kept as an
-- explicit, idempotent guarantee so the invariant lives next to the widening
-- above instead of depending on a 2022 migration file - and so a database whose
-- constraint never applied is repaired rather than silently left unprotected.
--
-- It deliberately REFUSES to proceed when duplicates exist: dropping user rows
-- from a migration would be data loss, so that decision belongs to a human.
--
-- NOTE: uniqueness here is case-SENSITIVE, because `username` is
-- `character varying`. ("Alice" and "alice" are distinct.) `users.email` is
-- `citext`, so it compares case-insensitively but carries no UNIQUE constraint
-- at all - see TESTING.md.
-- ---------------------------------------------------------------------------

DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conrelid = 'public.users'::regclass
          AND conname = 'users_name_key'
    ) THEN
        RAISE NOTICE 'users_name_key already present on public.users - no change needed';
        RETURN;
    END IF;

    IF EXISTS (
        SELECT 1
        FROM public.users
        GROUP BY username
        HAVING count(*) > 1
    ) THEN
        RAISE EXCEPTION
            'duplicate usernames found in public.users; resolve them before adding the UNIQUE constraint';
    END IF;

    ALTER TABLE public.users ADD CONSTRAINT users_name_key UNIQUE (username);
    RAISE NOTICE 'added users_name_key UNIQUE (username) to public.users';
END
$$;

-- ---------------------------------------------------------------------------
-- Restore `users.email` uniqueness.
--
-- `20220620050241_initial.sql:389` declares
--     ADD CONSTRAINT users_email_key UNIQUE (email);
-- but `email` is `citext`, and YugabyteDB refuses to index a
-- `user_defined_type` column:
--     ERROR: INDEX on column of type 'user_defined_type' not yet supported
-- so on YugabyteDB that constraint silently never applies and duplicate
-- accounts per address become possible.
--
-- A unique index over an EXPRESSION is accepted, so
-- `lower(email)` restores the intended invariant and keeps citext's
-- case-insensitive semantics. On vanilla PostgreSQL the constraint exists
-- already and the index name check below short-circuits, so this is a no-op.
--
-- The index deliberately reuses the name `users_email_key` so tooling that
-- looks the constraint up by name still finds it.
-- ---------------------------------------------------------------------------

DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM pg_indexes
        WHERE schemaname = 'public'
          AND tablename  = 'users'
          AND indexname  = 'users_email_key'
    ) THEN
        RAISE NOTICE 'users_email_key already present on public.users - no change needed';
        RETURN;
    END IF;

    IF EXISTS (
        SELECT 1 FROM public.users
        GROUP BY lower(email)
        HAVING count(*) > 1
    ) THEN
        RAISE EXCEPTION
            'duplicate emails found in public.users; resolve them before adding the UNIQUE index';
    END IF;

    CREATE UNIQUE INDEX users_email_key ON public.users (lower(email));
    RAISE NOTICE 'added users_email_key UNIQUE (lower(email)) to public.users';
END
$$;

-- ---------------------------------------------------------------------------
-- Restore `tags.name` uniqueness.
--
-- `20220620050241_initial.sql:371` declares
--     ADD CONSTRAINT tags_name_key UNIQUE (name);
-- but `tags.name` is `citext`, so on YugabyteDB it fails with the same
-- "INDEX on column of type 'user_defined_type' not yet supported" error.
--
-- Without it, `questions::dao::insert_question` rejects any question whose
-- tag list resolves to a different number of rows than tags requested, and a
-- duplicated tag row surfaces to the user as the misleading
-- "All tags were not found".
--
-- `lower(name)` restores the intended case-insensitive invariant.
-- ---------------------------------------------------------------------------

DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM pg_indexes
        WHERE schemaname = 'public'
          AND tablename  = 'tags'
          AND indexname  = 'tags_name_key'
    ) THEN
        RAISE NOTICE 'tags_name_key already present on public.tags - no change needed';
        RETURN;
    END IF;

    IF EXISTS (
        SELECT 1 FROM public.tags
        GROUP BY lower(name)
        HAVING count(*) > 1
    ) THEN
        RAISE EXCEPTION
            'duplicate tag names found in public.tags; resolve them before adding the UNIQUE index';
    END IF;

    CREATE UNIQUE INDEX tags_name_key ON public.tags (lower(name));
    RAISE NOTICE 'added tags_name_key UNIQUE (lower(name)) to public.tags';
END
$$;
