use crate::domain::uuid::Uuid;

pub struct NewTopicSubscriber {
    pub organizationId: Uuid,
    pub deviceId: Uuid,
}