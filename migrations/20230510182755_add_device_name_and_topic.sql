-- Adding additional columns that describes device and topic
ALTER TABLE subscriptions_topics
    ADD COLUMN device_name VARCHAR,
    ADD COLUMN topic_prefix VARCHAR;
