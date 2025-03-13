use std::io;
use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use actix_web::dev::Server;
use actix_web::middleware::Logger;
use lazy_static::lazy_static;
use prometheus::{register_counter, Counter, Encoder, TextEncoder};
use crate::config::Settings;

lazy_static! {
    pub (crate) static ref METRICS_TO_TEXT_ENCODER: TextEncoder = TextEncoder::new();
    pub (crate) static ref HTTP_COUNTER: Counter = register_counter!(
        "http_requests_total",
        "Total number of HTTP requests received"
    )
    .unwrap();
}

#[get("/metrics")]
async fn metrics_endpoint() -> impl Responder {
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    METRICS_TO_TEXT_ENCODER
        .encode(&metric_families, &mut buffer)
        .unwrap();
    let response = String::from_utf8(buffer).unwrap();
    HttpResponse::Ok()
        .content_type(METRICS_TO_TEXT_ENCODER.format_type())
        .body(response)
}

pub (crate) fn start_metrics_server(settings: &Settings) -> Result<Server, io::Error> {
    Ok(HttpServer::new(move || App::new().wrap(Logger::default()).service(metrics_endpoint))
        .shutdown_timeout(settings.graceful_shutdown_timeout_seconds)
        .bind(format!("0.0.0.0:{}", settings.metrics_port))?
        .run())
}
