use beesbuddy_bumblebee::configuration::get_configuration;
use beesbuddy_bumblebee::startup;
use beesbuddy_bumblebee::startup::Application;
use beesbuddy_bumblebee::telemetry::{get_subscriber, init_subscriber};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    App(#[from] startup::Error)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let subscriber = get_subscriber("bumblebee".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");
    let application = Application::build(configuration).await?;
    application.run_until_stopped().await?;
    Ok(())
}