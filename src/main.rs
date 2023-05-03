use beesbuddy_bumblebee::application::Application;
use beesbuddy_bumblebee::configuration::get_configuration;
use beesbuddy_bumblebee::listener::run_listener_until_stopped;
use beesbuddy_bumblebee::telemetry::{get_subscriber, init_subscriber};
use beesbuddy_bumblebee::worker::run_worker_until_stopped;
use beesbuddy_bumblebee::{application, utils};
use rumqttc::{AsyncClient, MqttOptions};
use std::fmt::Debug;
use std::time::Duration;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    App(#[from] application::Error),
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let subscriber = get_subscriber("bumblebee".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    let configuration = get_configuration().expect("Failed to read configuration.");

    let mut mqtt_options = MqttOptions::new(
        "beesbuddy-bumblebee", 
        configuration.clone().mqtt.host, 
        configuration.clone().mqtt.port,
    );
    mqtt_options.set_keep_alive(Duration::from_secs(10));
    let (async_client, event_loop) = AsyncClient::new(mqtt_options, 10);

    let application = Application::build(configuration.clone()).await?;
    let application_task = tokio::spawn(application.run_until_stopped());
    let worker_task = tokio::spawn(run_worker_until_stopped(
        configuration.clone(),
        rx,
        async_client,
        event_loop,
    ));
    let listener_task = tokio::spawn(run_listener_until_stopped(configuration, tx));

    tokio::select! {
        o = application_task => utils::report_exit("Web application", o),
        o = worker_task =>  utils::report_exit("Metrics delivery worker", o),
        o = listener_task =>  utils::report_exit("Table change listener", o),
    }

    Ok(())
}
