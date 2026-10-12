-- Fix `public.tokens_delete_old_rows()`.
--
-- `20220620050241_initial.sql:16-22` defines it as
--
--     CREATE FUNCTION public.tokens_delete_old_rows() RETURNS trigger
--       AS $$ BEGIN
--           DELETE FROM views WHERE created_date < NOW() - INTERVAL '5 minute';
--           RETURN NEW;
--       END; $$ LANGUAGE plpgsql;
--
-- i.e. a copy-paste of `views_delete_old_rows()`. The token expiry trigger
-- therefore purged rows from `views` on every token insert and never from
-- `tokens`, so expired tokens accumulated forever.
--
-- On PostgreSQL this is merely wrong; on YugabyteDB the `views` DELETE raises
-- SQLSTATE 40001 "schema version mismatch" from inside the trigger, which
-- aborted the enclosing INSERT and made `register` / `submit-email` answer 500.
--
-- `CREATE OR REPLACE` keeps the function OID, so both existing triggers keep
-- working: `expire_tokens_delete_old_rows_trigger` (AFTER INSERT, per
-- statement) and `trg_tokens_delete_old_rows` (BEFORE INSERT, per row).
-- A DELETE does not re-fire an INSERT trigger, so there is no recursion.
--
-- The initial migration is left untouched because it is already applied.

CREATE OR REPLACE FUNCTION public.tokens_delete_old_rows() RETURNS trigger
  AS $$
  BEGIN
    DELETE FROM tokens WHERE created_date < NOW() - INTERVAL '5 minute';
    RETURN NEW;
  END;
$$ LANGUAGE plpgsql;
