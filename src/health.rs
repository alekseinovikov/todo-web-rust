use actix_web::{HttpResponse, Responder, get, web};

#[get("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().body("Healthy")
}

#[get("/ready")]
async fn readiness() -> impl Responder {
    HttpResponse::Ok().body("Ready")
}

pub(crate) fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(health).service(readiness);
}
