-- Make the token-expiry purge safe on YugabyteDB.
--
-- `20261001161000_fix_tokens_delete_old_rows.sql` fixed the purge to delete
-- from `tokens`, but it still runs a table-wide
--
--     DELETE FROM tokens WHERE created_date < NOW() - INTERVAL '5 minute';
--
-- inside both token triggers (`trg_tokens_delete_old_rows`, BEFORE INSERT
-- FOR EACH ROW, and `expire_tokens_delete_old_rows_trigger`, AFTER INSERT
-- FOR EACH STATEMENT). Because the purge scans the whole `tokens` table
-- from inside the INSERT's own transaction, two concurrent token writers
-- (e.g. `register` and `submit-email` racing, as the REST harness does)
-- hit SQLSTATE 40001:
--
--     could not serialize access due to concurrent update
--     (query layer retries aren't supported when executing non-first
--     statement in batch, will be unable to replay earlier commands)
--
-- which aborted the enclosing INSERT and surfaced to the user as a 500 on
-- `POST /api/v1/register`.
--
-- Catching the error inside a plpgsql sub-block is not a fix: YugabyteDB
-- cannot replay a failed statement mid-batch, and the transaction stays
-- poisoned. Instead this migration serializes token writes with a
-- transaction-scoped advisory lock. Any session that inserts into `tokens`
-- takes the lock first, so the purge in one transaction can never overlap
-- the insert in another — they run strictly in sequence and each purge
-- reads the committed state.
--
-- `pg_advisory_xact_lock` is reentrant within the same transaction, so the
-- row-level and statement-level triggers firing back-to-back in one INSERT
-- acquire it once. The lock is released at COMMIT/ROLLBACK; a crash cannot
-- leak it.

CREATE OR REPLACE FUNCTION public.tokens_delete_old_rows() RETURNS trigger
  AS $$
  BEGIN
    -- Serialize all token writes. Without this the table-wide purge below
    -- conflicts with concurrent inserts (YB SQLSTATE 40001) and fails the
    -- caller's INSERT.
    PERFORM pg_advisory_xact_lock(hashtext('tokens_delete_old_rows'));

    DELETE FROM tokens WHERE created_date < NOW() - INTERVAL '5 minute';

    RETURN NEW;
  END;
  $$ LANGUAGE plpgsql;
