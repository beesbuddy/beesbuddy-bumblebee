-- Add UPDATE row trigger
DROP TRIGGER IF EXISTS subscriptions_topics_notify_update ON subscriptions_topics;
CREATE TRIGGER subscriptions_topics_notify_update AFTER UPDATE ON subscriptions_topics FOR EACH ROW EXECUTE PROCEDURE subscriptions_topics_update_notify();

-- Add INSERT row trigger
DROP TRIGGER IF EXISTS subscriptions_topics_notify_insert ON subscriptions_topics;
CREATE TRIGGER subscriptions_topics_notify_insert AFTER INSERT ON subscriptions_topics FOR EACH ROW EXECUTE PROCEDURE subscriptions_topics_update_notify();

-- Add DELETE row trigger
DROP TRIGGER IF EXISTS subscriptions_topics_notify_delete ON subscriptions_topics;
CREATE TRIGGER subscriptions_topics_notify_delete AFTER DELETE ON subscriptions_topics FOR EACH ROW EXECUTE PROCEDURE subscriptions_topics_update_notify();
