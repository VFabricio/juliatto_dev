CREATE TYPE subscription_status AS ENUM ('waiting_validation', 'active', 'unsubscribed');

CREATE TABLE subscriptions (
  id uuid PRIMARY KEY DEFAULT uuidv7(),
  name text NOT NULL,
  email text NOT NULL UNIQUE,
  status subscription_status NOT NULL,
  verification_code text NOT NULL,
  unsubscription_code text NOT NULL
);

SELECT trigger_updated_at('subscriptions');
