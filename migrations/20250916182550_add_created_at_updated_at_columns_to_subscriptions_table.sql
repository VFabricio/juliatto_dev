ALTER TABLE subscriptions
  ADD COLUMN created_at timestamptz,
  ADD COLUMN updated_at timestamptz;

UPDATE subscriptions
  SET created_at = now();

ALTER TABLE subscriptions
  ALTER COLUMN created_at SET NOT NULL,
  ALTER COLUMN created_at SET DEFAULT now();
