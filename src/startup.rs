use crate::configuration::{DatabaseSettings, Settings};
use crate::routes::{get_admin_dashboard, get_view_admin_subscriptions_topics, get_create_admin_subscriptions_topics, health_check, home, post_create_admin_subscriptions_topics};
use actix_web::dev::Server;
use actix_web::web::Data;
use actix_web::{web, App, HttpServer};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::fs;
use std::net::TcpListener;
use std::path::PathBuf;
use std::thread::scope;
use actix_web::cookie::Key;
use actix_web_flash_messages::FlashMessagesFramework;
use actix_web_flash_messages::storage::CookieMessageStore;
use secrecy::{ExposeSecret, Secret};
use tracing_actix_web::TracingLogger;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Filesystem error for `{0}`: `{1}`")]
    Io(PathBuf, std::io::Error),
    #[error(transparent)]
    Startup(#[from] std::io::Error),
}

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, Error> {
        let connection_pool = get_connection_pool(&configuration.database);

        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr().unwrap().port();
        let server = run(
            listener,
            connection_pool,
            configuration.application.base_url,
            configuration.application.web_dir_path,
            configuration.application.hmac_secret,
        )?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), Error> {
        self.server.await.map_err(|e| Error::Startup(e))
    }
}

pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.with_db())
}

pub struct ApplicationBaseUrl(pub String);

fn run(
    listener: TcpListener,
    db_pool: PgPool,
    base_url: String,
    web_dir_path: String,
    hmac_secret: Secret<String>,
) -> Result<Server, Error> {
    let db_pool = Data::new(db_pool);
    let base_url = Data::new(ApplicationBaseUrl(base_url));

    fs::create_dir_all(web_dir_path.as_str())
        .map_err(|e| Error::Io(web_dir_path.parse().unwrap(), e))?;

    let secret_key = Key::from(hmac_secret.expose_secret().as_bytes());
    let message_store = CookieMessageStore::builder(secret_key.clone()).build();
    let message_framework = FlashMessagesFramework::builder(message_store).build();

    // Create a logic that will create a dir if it doesn't exist.
    // Initialize PathBuf from string and use in actix_file
    let server = HttpServer::new(move || {
        App::new()
            .wrap(message_framework.clone())
            .wrap(TracingLogger::default())
            .route("/", web::get().to(home))
            .service(web::scope("/admin")
                .route("/dashboard", web::get().to(get_admin_dashboard))
                .service(web::scope("/subscriptions")
                    .route("/topics/view", web::get().to(get_view_admin_subscriptions_topics))
                    .route("/topics/create", web::post().to(post_create_admin_subscriptions_topics))
                    .route("/topics/create", web::get().to(get_create_admin_subscriptions_topics))
                )
            )
            .route("/health_check", web::get().to(health_check))
            .service(
                actix_files::Files::new("/web", web_dir_path.as_str())
                    .redirect_to_slash_directory()
                    .index_file("index.html"),
            )
            .app_data(db_pool.clone())
            .app_data(base_url.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}
