use actix_web::{http::header::ContentType, web, HttpResponse};
use handlebars::Handlebars;

pub async fn home(hb: web::Data<Handlebars<'_>>) -> HttpResponse {
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(hb.render("home", &()).unwrap())
}
