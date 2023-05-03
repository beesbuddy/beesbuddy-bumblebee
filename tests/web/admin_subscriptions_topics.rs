use uuid::Uuid;
use crate::helpers::{assert_is_redirect_to, spawn_app};

#[tokio::test]
async fn admin_subscriptions_topics_view_all_returns_a_200() {
    let app = spawn_app().await;

    let html_page = app.get_admin_subscriptions_topics_html().await;
    assert!(html_page.contains("Available topics:"));
}

#[tokio::test]
async fn admin_subscriptions_topics_add_new_and_return_a_200() {
    let app = spawn_app().await;

    let organization_id = Uuid::new_v4().to_string();
    let device_id = Uuid::new_v4().to_string();

    let body = serde_urlencoded::to_string(&serde_json::json!({
        "organization_id": organization_id,
        "device_id": device_id
    })).unwrap();

    let response = app.post_subscriptions_topics(body)
        .await
        .error_for_status()
        .unwrap();

    assert_is_redirect_to(&response, "/admin/subscriptions/topics/view")
}
