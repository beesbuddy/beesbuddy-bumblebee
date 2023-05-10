-- Add not null constrain
TRUNCATE TABLE subscriptions_topics;

ALTER TABLE subscriptions_topics
    ALTER COLUMN device_name SET NOT NULL;
ALTER TABLE subscriptions_topics
    ALTER COLUMN topic_prefix SET NOT NULL;
