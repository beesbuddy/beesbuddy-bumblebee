use crate::domain::id::Id;

pub struct NewTopicSubscriber {
    pub organizationId: Id,
    pub deviceId: Id,
}