use crate::domain::{Id, NewSubscriberTopic, ViewSubscriberTopic};
use crate::utils::{error_chain_fmt, see_other};
use actix_web::http::StatusCode;
use actix_web::{http::header::ContentType, web, HttpResponse, ResponseError};
use actix_web_flash_messages::{FlashMessage, IncomingFlashMessages};
use anyhow::Context;
use chrono::Utc;
use sqlx::{PgPool, Postgres, Transaction};
use std::convert::{TryFrom, TryInto};
use std::fmt::Write;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    device_id: String,
    organization_id: String,
}

impl TryFrom<FormData> for NewSubscriberTopic {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let device_id = Id::parse(value.device_id)?;
        let organization_id = Id::parse(value.organization_id)?;
        Ok(Self {
            organization_id,
            device_id,
        })
    }
}

#[derive(thiserror::Error)]
pub enum TopicSubscribeError {
    #[error("{0}")]
    ValidationError(String),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for TopicSubscribeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for TopicSubscribeError {
    fn status_code(&self) -> StatusCode {
        match self {
            TopicSubscribeError::ValidationError(_) => StatusCode::BAD_REQUEST,
            TopicSubscribeError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

pub async fn get_view_admin_subscriptions_topics(
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, TopicSubscribeError> {
    let topics = select_subscribers_topics(&pool).await.unwrap();

    let mut topics_html = String::new();

    for topic in topics {
        writeln!(
            topics_html,
            "<p><i>{} - {} </i></p>",
            topic.organization_id, topic.device_id
        )
        .unwrap();
    }

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta http-equiv="content-type" content="text/html; charset=utf-8">
    <title>View subscriptions</title>
</head>
<body>
    <a href="/admin/subscriptions/topics/create">Create a new topic</a>
    <p>Available topics:</p>
    {topics_html}
</body>
</html>"#
        )))
}

pub async fn get_create_admin_subscriptions_topics(
    flash_messages: IncomingFlashMessages,
) -> Result<HttpResponse, actix_web::Error> {
    let mut msg_html = String::new();

    for m in flash_messages.iter() {
        writeln!(msg_html, "<p><i>{}</i></p>", m.content()).unwrap();
    }

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta http-equiv="content-type" content="text/html; charset=utf-8">
    <title>Create a subscription</title>
</head>
<body>
    {msg_html}
    <p>Create subscription for topic</p>
    <form action="/admin/subscriptions/topics/create" method="post">
        <div style="margin-bottom: 5px">
            <label>Organization id:<br>
                <input
                    type="text"
                    placeholder="Enter organization id"
                    name="organization_id"
                >
            </label>
        </div>
        <div style="margin-bottom: 5px">
             <label>Device id:<br>
                <input
                    type="text"
                    placeholder="Enter device id"
                    name="device_id"
                >
            </label>
        </div>
        <div>
            <button type="submit">Subscribe</button>
        </div>
    </form>
</body>
</html>"#,
        )))
}

#[tracing::instrument(
name = "Adding a new subscriber",
skip(form, pool),
fields(
organization_id = % form.organization_id,
device_id = % form.device_id
)
)]
pub async fn post_create_admin_subscriptions_topics(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, TopicSubscribeError> {
    let new_subscriber: NewSubscriberTopic = form
        .0
        .try_into()
        .map_err(TopicSubscribeError::ValidationError)?;
    let mut transaction = pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool")?;
    insert_subscriber_topic(&mut transaction, &new_subscriber)
        .await
        .context("Failed to insert new subscriber in the database.")?;
    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to store a new subscriber.")?;

    FlashMessage::info("Your device has been added to subscribe list.").send();
    Ok(see_other("/admin/subscriptions/topics/view"))
}

#[tracing::instrument(name = "Select all subscribers from the database", skip(pool))]
pub async fn select_subscribers_topics(
    pool: &PgPool,
) -> Result<Vec<ViewSubscriberTopic>, sqlx::Error> {
    let subscribers = sqlx::query_as!(
        ViewSubscriberTopic,
        r#"
    SELECT organization_id, device_id FROM subscriptions_topics
    "#
    )
    .fetch_all(pool)
    .await
    .context("Failed to performed a query to retrieve stored credentials.")
    .unwrap();

    Ok(subscribers)
}

#[tracing::instrument(
    name = "Insert a new subscriber topic in the database",
    skip(new_subscriber, transaction)
)]
pub async fn insert_subscriber_topic(
    transaction: &mut Transaction<'_, Postgres>,
    new_subscriber: &NewSubscriberTopic,
) -> Result<Uuid, sqlx::Error> {
    let subscriber_id = Uuid::new_v4();
    sqlx::query!(
        r#"
    INSERT INTO subscriptions_topics (id, organization_id, device_id, created_at, updated_at)
    VALUES ($1, $2, $3, $4, $5)
            "#,
        subscriber_id,
        new_subscriber.organization_id.as_ref(),
        new_subscriber.device_id.as_ref(),
        Utc::now(),
        Utc::now()
    )
    .execute(transaction)
    .await?;
    Ok(subscriber_id)
}
