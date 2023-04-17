use actix_web::{http::header::ContentType, HttpResponse};

pub async fn get_admin_dashboard() -> Result<HttpResponse, actix_web::Error> {
    Ok(HttpResponse::Ok().content_type(ContentType::html()).body(
        r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta http-equiv="content-type" content="text/html; charset=utf-8">
    <title>Admin dashboard</title>
</head>
<body>
    <p>Welcome!</p>
    <p>Available actions:</p>
    <ol>
        <li><a href="/admin/subscriptions/topics/view">View topics</a></li>
    </ol>
</body>
</html>"#
            .to_string(),
    ))
}
