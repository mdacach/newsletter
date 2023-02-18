-- By wrapping the migration in a transaction we make sure that everything succeeds
-- or fails together.
BEGIN;
-- For all previous subscribers (where we did not have the `status` column)
-- we update their status.
UPDATE subscriptions
SET status = 'confirmed'
WHERE status IS NULL;
-- After that, we can make `status` NOT NULL.
ALTER TABLE subscriptions
    ALTER COLUMN status SET NOT NULL;
COMMIT;