-- Add a table update notification function
CREATE OR REPLACE FUNCTION subscriptions_topics_update_notify() RETURNS trigger AS $$
DECLARE
  id UUID;
  organization_id UUID;
  device_id UUID;
BEGIN
  IF TG_OP = 'INSERT' OR TG_OP = 'UPDATE' THEN
    id = NEW.id;
    organization_id = NEW.organization_id;
    device_id = NEW.device_id;
  ELSE
    id = OLD.id;
    organization_id = OLD.organization_id;
    device_id = OLD.device_id;
  END IF;
  PERFORM pg_notify('subscriptions_topics', json_build_object('table', TG_TABLE_NAME, 'id', id, 'organization_id', organization_id, 'device_id', device_id, 'action_type', TG_OP)::text);
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;
