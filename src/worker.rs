use crate::application::get_connection_pool;
use crate::configuration::Settings;
use crate::listener::{ActionType, SubscriptionTopicsNotificationPayload};
use crate::utils;
use log::warn;
use rumqttc::{AsyncClient, Event, EventLoop, Incoming, QoS};
use sqlx::PgPool;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedReceiver;
use tracing::{error, info};

// TODO: Make it more generic
pub async fn run_worker_until_stopped(
    configuration: Settings,
    rx: UnboundedReceiver<SubscriptionTopicsNotificationPayload>,
    client: AsyncClient,
    event_loop: EventLoop,
) -> Result<(), anyhow::Error> {
    let connection_pool = get_connection_pool(&configuration.database);
    worker_loop(connection_pool, rx, client, event_loop).await
}

#[tracing::instrument(name = "Worker loop", skip(pool, rx, client, event_loop))]
async fn worker_loop(
    pool: PgPool,
    rx: UnboundedReceiver<SubscriptionTopicsNotificationPayload>,
    mut client: AsyncClient,
    event_loop: EventLoop,
) -> Result<(), anyhow::Error> {
    setup_initial_subscribers(pool, &mut client).await.unwrap();

    let notification_receiver = tokio::spawn(run_message_receiver(event_loop));
    let subscriptions_change_listener = tokio::spawn(run_subscriptions_change_listener(rx, client));

    tokio::select! {
        o = notification_receiver => utils::report_exit("Notification receiver", o),
        o = subscriptions_change_listener =>  utils::report_exit("Subscriptions change listener", o),
    }

    Ok(())
}

#[tracing::instrument(name = "Receiving message", skip(event_loop))]
async fn run_message_receiver(mut event_loop: EventLoop) -> Result<(), anyhow::Error> {
    loop {
        let event = event_loop.poll().await;
        match &event {
            Ok(notification) => match notification {
                Event::Incoming(incoming) => {
                    if let Incoming::Publish(publish) = incoming {
                        let payload = String::from_utf8(publish.payload.to_vec());
                        info!("{payload:?}");
                    }
                }
                Event::Outgoing(_) => {}
            },
            Err(error) => {
                warn!("Error = {error:?}");
                tokio::time::sleep(Duration::from_secs(3)).await;
            }
        }
    }
}

#[tracing::instrument(name = "Receiving subscriptions changes", skip(rx, client))]
async fn run_subscriptions_change_listener(
    mut rx: UnboundedReceiver<SubscriptionTopicsNotificationPayload>,
    client: AsyncClient,
) -> Result<(), anyhow::Error> {
    loop {
        let payload = rx.recv().await.unwrap();
        info!("received data {:?}", payload);

        match payload.action_type {
            ActionType::INSERT => {
                match client.try_subscribe(
                    format!(
                        "apiary/{}/hive/{}",
                        payload.organization_id, payload.device_id
                    ),
                    QoS::AtLeastOnce,
                ) {
                    Ok(_) => info!(
                        "added subscription: apiary/{}/hive/{}",
                        payload.organization_id, payload.device_id
                    ),
                    Err(err) => {
                        error!("error on adding subscription: {err:?}")
                    }
                }
            }
            ActionType::DELETE => {
                match client.try_unsubscribe(format!(
                    "apiary/{}/hive/{}",
                    payload.organization_id, payload.device_id
                )) {
                    Ok(_) => info!(
                        "removed subscription: apiary/{}/hive/{}",
                        payload.organization_id, payload.device_id
                    ),
                    Err(err) => {
                        error!("error on removing subscription: {err:?}")
                    }
                }
            }
            _ => {
                info!("not supported action");
            }
        };
    }
}

async fn setup_initial_subscribers(
    pool: PgPool,
    client: &mut AsyncClient,
) -> Result<(), anyhow::Error> {
    let mut transaction = pool.begin().await?;

    let subscriptions = sqlx::query!(
        r#"
        SELECT organization_id, device_id
            FROM subscriptions_topics
        "#,
    )
    .fetch_all(&mut transaction)
    .await?;

    for subscription in subscriptions {
        match client.try_subscribe(
            format!(
                "apiary/{}/hive/{}",
                subscription.organization_id, subscription.device_id
            ),
            QoS::AtLeastOnce,
        ) {
            Ok(_) => info!(
                "added subscription: apiary/{}/hive/{}",
                subscription.organization_id, subscription.device_id
            ),
            Err(err) => error!("error on adding subscription: {err:?}"),
        }
    }

    Ok(())
}
