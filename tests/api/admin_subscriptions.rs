use crate::helpers::spawn_app;

#[tokio::test]
async fn admin_subscriptions_returns_a_200_for_main_page() {
    let app = spawn_app().await;

    let html_page = app.get_admin_subscriptions_html().await;
    assert!(html_page.contains("Available subscriptions:"));
}
