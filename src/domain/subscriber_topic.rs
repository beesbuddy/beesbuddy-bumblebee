use crate::domain::id::Id;

#[derive(Debug, Clone)]
pub struct NewSubscriberTopic {
    pub organization_id: Id,
    pub device_id: Id,
}

#[derive(Debug, Clone)]
pub struct ViewSubscriberTopic {
    pub organization_id: uuid::Uuid,
    pub device_id: uuid::Uuid,
}
