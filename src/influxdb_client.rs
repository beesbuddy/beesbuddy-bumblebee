use crate::domain::HiveData;
use anyhow::anyhow;
use log::{debug, info, warn};
use reqwest::Client;
use secrecy::{ExposeSecret, Secret};

#[derive(Clone)]
pub struct InfluxDbClient {
    http_client: Client,
    base_url: String,
    bucket: String,
    organization: String,
    authorization_token: Secret<String>,
}

impl InfluxDbClient {
    pub fn new(
        base_url: String,
        bucket: String,
        organization: String,
        authorization_token: Secret<String>,
        timeout: std::time::Duration,
    ) -> Self {
        let http_client = Client::builder().timeout(timeout).build().unwrap();
        Self {
            http_client,
            base_url,
            bucket,
            organization,
            authorization_token,
        }
    }

    pub async fn write(&self, payload: &str) -> Result<(), anyhow::Error> {
        let url = format!(
            "{}/api/v2/write?org={}&bucket={}",
            self.base_url, self.organization, self.bucket
        );

        info!("data point to store in influxdb: {payload:?}");

        let content = self
            .http_client
            .post(&url)
            .header(
                "Authorization",
                format!("Token {}", self.authorization_token.expose_secret()),
            )
            .header("Content-Type", "text/plain; charset=utf-8")
            .header("Accept", "application/json")
            .body(payload.to_string())
            .send()
            .await?
            .text()
            .await?;

        info!("{content:?}");

        match self
            .http_client
            .post(&url)
            .header(
                "Authorization",
                format!("Token {}", self.authorization_token.expose_secret()),
            )
            .header("Content-Type", "text/plain; charset=utf-8")
            .header("Accept", "application/json")
            .body(payload.to_string())
            .send()
            .await?
            .error_for_status()
        {
            Ok(_) => Ok(()),
            Err(err) => {
                warn!("Error during point line data send = {err:?}");
                anyhow::bail!("Error during point line data send = {err:?}")
            }
        }
    }
}
