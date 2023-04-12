use actix_web::{http::header::ContentType, HttpResponse};

pub async fn admin_subscriptions() -> Result<HttpResponse, actix_web::Error> {
    Ok(HttpResponse::Ok().content_type(ContentType::html()).body(
        r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta http-equiv="content-type" content="text/html; charset=utf-8">
    <title>Admin dashboard</title>
</head>
<body>
    <p>Available subscriptions:</p>
</body>
</html>"#
            .to_string(),
    ))
}
