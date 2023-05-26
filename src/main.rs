use beesbuddy_bumblebee::application::Application;
use beesbuddy_bumblebee::configuration::get_configuration;
use beesbuddy_bumblebee::telemetry::{get_subscriber, init_subscriber};
use beesbuddy_bumblebee::workers::{
    run_mqtt_worker_until_stopped, run_subscription_worker_until_stopped,
};
use beesbuddy_bumblebee::{application, utils};
use rumqttc::tokio_rustls::rustls;
use rumqttc::tokio_rustls::rustls::ClientConfig;
use rumqttc::{AsyncClient, MqttOptions, Transport};
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
        format!("beesbuddy-bumblebee-{}", uuid::Uuid::new_v4()),
        configuration.clone().mqtt.host,
        configuration.clone().mqtt.port,
    );
    mqtt_options.set_keep_alive(Duration::from_secs(5));
    mqtt_options.set_credentials(
        configuration.clone().mqtt.username,
        configuration.clone().mqtt.password,
    );
    mqtt_options.set_clean_session(true);

    if configuration.clone().mqtt.port == 8883 {
        let mut root_cert_store = rustls::RootCertStore::empty();
        for cert in rustls_native_certs::load_native_certs().expect("could not load platform certs")
        {
            root_cert_store
                .add(&rustls::Certificate(cert.0))
                .expect("unable to add certs");
        }

        let client_config = ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_cert_store)
            .with_no_client_auth();

        mqtt_options.set_transport(Transport::tls_with_config(client_config.into()));
    }

    let application = Application::build(configuration.clone()).await?;
    let application_task = tokio::spawn(application.run_until_stopped());

    let (mqtt_client, mqtt_event_loop) = AsyncClient::new(mqtt_options, 20);
    let mqtt_worker_task = tokio::spawn(run_mqtt_worker_until_stopped(
        configuration.clone(),
        rx,
        mqtt_client,
        mqtt_event_loop,
    ));

    let subscriptions_worker_task =
        tokio::spawn(run_subscription_worker_until_stopped(configuration, tx));

    tokio::select! {
        o = application_task => utils::report_exit("Web application", o),
        o = mqtt_worker_task =>  utils::report_exit("Metrics/mqtt delivery worker", o),
        o = subscriptions_worker_task =>  utils::report_exit("Table/subscriptions change listener", o),
    }

    Ok(())
}
