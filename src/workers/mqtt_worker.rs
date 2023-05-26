use crate::application::get_connection_pool;
use crate::configuration::Settings;
use crate::domain::HiveData;
use crate::influxdb_client::InfluxDbClient;
use crate::utils;
use crate::workers::{ActionType, SubscriptionTopicsNotificationPayload};
use log::warn;
use rumqttc::{AsyncClient, Event, EventLoop, Incoming, QoS};
use sqlx::PgPool;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedReceiver;
use tracing::{error, info};

pub async fn run_mqtt_worker_until_stopped(
    configuration: Settings,
    rx: UnboundedReceiver<SubscriptionTopicsNotificationPayload>,
    mqtt_client: AsyncClient,
    mqtt_event_loop: EventLoop,
) -> Result<(), anyhow::Error> {
    let connection_pool = get_connection_pool(&configuration.database);
    let influxdb_client = configuration.influxdb.client();
    mqtt_worker_loop(
        connection_pool,
        rx,
        mqtt_client,
        mqtt_event_loop,
        influxdb_client,
    )
    .await
}

#[tracing::instrument(
    name = "Mqtt worker loop",
    skip(db_pool, rx, client, event_loop, influxdb_client)
)]
async fn mqtt_worker_loop(
    db_pool: PgPool,
    rx: UnboundedReceiver<SubscriptionTopicsNotificationPayload>,
    client: AsyncClient,
    event_loop: EventLoop,
    influxdb_client: InfluxDbClient,
) -> Result<(), anyhow::Error> {
    setup_initial_subscribers(db_pool.clone(), client.clone())
        .await
        .unwrap();

    let notification_receiver = tokio::spawn(run_message_processor(
        db_pool.clone(),
        client.clone(),
        event_loop,
        influxdb_client,
    ));
    let subscriptions_change_listener =
        tokio::spawn(run_subscriptions_change_listener(rx, client.clone()));

    tokio::select! {
        o = notification_receiver => utils::report_exit("Notification receiver", o),
        o = subscriptions_change_listener =>  utils::report_exit("Subscriptions change listener", o),
    }

    Ok(())
}

#[tracing::instrument(
    name = "Processing mqtt message",
    skip(db_pool, mqtt_client, mqtt_event_loop, influxdb_client)
)]
async fn run_message_processor(
    db_pool: PgPool,
    mqtt_client: AsyncClient,
    mut mqtt_event_loop: EventLoop,
    influxdb_client: InfluxDbClient,
) -> Result<(), anyhow::Error> {
    loop {
        let event = mqtt_event_loop.poll().await;

        match &event {
            Ok(notification) => match notification {
                Event::Incoming(incoming) => {
                    if let Incoming::Publish(publish) = incoming {
                        match HiveData::try_from(publish.payload.to_vec()) {
                            Ok(data) => {
                                let _ = influxdb_client
                                    .write(data.format_line_point().as_str())
                                    .await;
                            }
                            Err(err) => {
                                error!("Error during raw payload reading = {err:?}");
                            }
                        }
                    }
                }
                Event::Outgoing(_) => {}
            },
            Err(error) => {
                error!("Error during message receive = {error:?}");
                tokio::time::sleep(Duration::from_secs(60)).await;
                match setup_initial_subscribers(db_pool.clone(), mqtt_client.clone()).await {
                    Ok(_) => info!("Initialized subscribers again"),
                    Err(error) => error!("Error during subscribers initializing = {error:?}"),
                }
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
                    format!("{}/{}", payload.topic_prefix, payload.device_name),
                    QoS::AtLeastOnce,
                ) {
                    Ok(_) => info!(
                        "added subscription: {}/{}",
                        payload.topic_prefix, payload.device_name
                    ),
                    Err(err) => {
                        error!("error on adding subscription: {err:?}")
                    }
                }
            }
            ActionType::DELETE => {
                match client
                    .try_unsubscribe(format!("{}/{}", payload.topic_prefix, payload.device_name))
                {
                    Ok(_) => info!(
                        "removed subscription: {}/{}",
                        payload.topic_prefix, payload.device_name
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

async fn setup_initial_subscribers(pool: PgPool, client: AsyncClient) -> Result<(), anyhow::Error> {
    let mut transaction = pool.begin().await?;

    match sqlx::query!(
        r#"
        SELECT topic_prefix, device_name
            FROM subscriptions_topics
        "#,
    )
    .fetch_all(&mut transaction)
    .await
    {
        Ok(subscriptions) => {
            for subscription in subscriptions {
                match client.try_subscribe(
                    format!("{}/{}", subscription.topic_prefix, subscription.device_name),
                    QoS::AtLeastOnce,
                ) {
                    Ok(_) => info!(
                        "added subscription: {}/{}",
                        subscription.topic_prefix, subscription.device_name
                    ),
                    Err(err) => warn!("error on adding subscription: {err:?}"),
                }
            }
        }
        Err(err) => {
            warn!("Not able to initialize subscriptions = {err:?}")
        }
    }

    Ok(())
}
