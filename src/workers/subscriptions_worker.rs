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

#[tracing::instrument(name = "Subscription worker loop", skip(configuration, tx))]
pub async fn run_subscription_worker_until_stopped(
    configuration: Settings,
    tx: UnboundedSender<SubscriptionTopicsNotificationPayload>,
) -> Result<(), anyhow::Error> {
    let connection_pool = get_connection_pool(&configuration.database);
    subscription_worker_loop(&connection_pool, tx).await
}

async fn subscription_worker_loop(
    db_pool: &PgPool,
    tx: UnboundedSender<SubscriptionTopicsNotificationPayload>,
) -> Result<(), anyhow::Error> {
    let mut listener = PgListener::connect_with(db_pool).await.unwrap();
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
