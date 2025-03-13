use actix_web::{HttpResponse, Responder, get, web};
use mime_guess::from_path;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "static/"]
pub struct Asset;

#[get("/")]
async fn index() -> impl Responder {
    match Asset::get("index.html") {
        Some(content) => HttpResponse::Ok()
            .content_type("text/html")
            .body(content.data.into_owned()),
        None => HttpResponse::NotFound().body("File is not found"),
    }
}

#[get("/static/{filename:.*}")]
async fn static_files(filename: web::Path<String>) -> impl Responder {
    match Asset::get(&filename) {
        Some(content) => {
            let body = content.data.into_owned();
            let mime = from_path(filename.into_inner()).first_or_octet_stream();
            HttpResponse::Ok().content_type(mime.as_ref()).body(body)
        }
        None => HttpResponse::NotFound().body("File is not found"),
    }
}

pub(crate) fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(index).service(static_files);
}
