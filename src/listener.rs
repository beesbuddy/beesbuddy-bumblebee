use crate::application::get_connection_pool;
use crate::configuration::Settings;
use serde::Deserialize;
use sqlx::postgres::PgListener;
use sqlx::PgPool;
use std::fmt::Debug;
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub enum ActionType {
    INSERT,
    UPDATE,
    DELETE,
}

#[derive(Deserialize, Debug)]
pub struct SubscriptionTopicsNotificationPayload {
    pub table: String,
    pub action_type: ActionType,
    pub organization_id: Uuid,
    pub device_id: Uuid,
    pub device_name: String,
    pub topic_prefix: String,
}

#[tracing::instrument(name = "Listener loop", skip(configuration, tx))]
pub async fn run_listener_until_stopped(
    configuration: Settings,
    tx: UnboundedSender<SubscriptionTopicsNotificationPayload>,
) -> Result<(), anyhow::Error> {
    let connection_pool = get_connection_pool(&configuration.database);
    listener_loop(&connection_pool, tx).await
}

// TODO: Make it more generic so it would be possible to track changes for any table with any payload
async fn listener_loop(
    pool: &PgPool,
    tx: UnboundedSender<SubscriptionTopicsNotificationPayload>,
) -> Result<(), anyhow::Error> {
    let mut listener = PgListener::connect_with(pool).await.unwrap();
    listener.listen_all(vec!["subscriptions_topics"]).await?;

    loop {
        while let Some(notification) = listener.try_recv().await? {
            let raw_payload = notification.payload().to_owned();
            let payload =
                serde_json::from_str::<SubscriptionTopicsNotificationPayload>(&raw_payload)
                    .unwrap();
            tx.send(payload).unwrap();
        }
    }
}
