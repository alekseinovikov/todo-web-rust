use actix_web::dev::Service;
use actix_web::middleware::Logger;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use config::{Config, ConfigError, Environment, File};
use futures::try_join;
use lazy_static::lazy_static;
use log::info;
use mime_guess::from_path;
use prometheus::{register_counter, Counter, Encoder, TextEncoder};

use serde::Deserialize;
use std::env;
mod embed;

lazy_static! {
    static ref METRICS_TO_TEXT_ENCODER: TextEncoder = TextEncoder::new();
    static ref HTTP_COUNTER: Counter = register_counter!(
        "http_requests_total",
        "Total number of HTTP requests received"
    )
    .unwrap();
}

#[derive(Debug, Deserialize)]
struct Settings {
    logger: LoggerSettings,
    graceful_shutdown_timeout_seconds: u64,
}

#[derive(Debug, Deserialize)]
struct LoggerSettings {
    level: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            logger: LoggerSettings {
                level: "info".to_string(),
            },
            graceful_shutdown_timeout_seconds: 10,
        }
    }
}

fn load_config() -> Result<Settings, ConfigError> {
    let config = Config::builder()
        // defaults
        .set_default("logger.level", "info")?
        .set_default("graceful_shutdown_timeout_seconds", 10)?
        // source file
        .add_source(File::with_name("config").required(false))
        // override values from ENV with APP_ prefix
        .add_source(Environment::with_prefix("APP").separator("__"))
        .build()?;
    config.try_deserialize()
}

#[get("/")]
async fn index() -> impl Responder {
    match embed::Asset::get("index.html") {
        Some(content) => HttpResponse::Ok()
            .content_type("text/html")
            .body(content.data.into_owned()),
        None => HttpResponse::NotFound().body("File is not found"),
    }
}

#[get("/static/{filename:.*}")]
async fn static_files(filename: web::Path<String>) -> impl Responder {
    match embed::Asset::get(&filename) {
        Some(content) => {
            let body = content.data.into_owned();
            let mime = from_path(filename.into_inner()).first_or_octet_stream();
            HttpResponse::Ok().content_type(mime.as_ref()).body(body)
        }
        None => HttpResponse::NotFound().body("File is not found"),
    }
}

#[get("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().body("Healthy")
}

#[get("/ready")]
async fn readiness() -> impl Responder {
    HttpResponse::Ok().body("Ready")
}

#[get("/metrics")]
async fn metrics() -> impl Responder {
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let settings = load_config().unwrap_or_default();
    if env::var("RUST_LOG").is_err() {
        unsafe {
            env::set_var("RUST_LOG", settings.logger.level.clone());
        }
    }

    env_logger::init();
    info!("Starting server at http://127.0.0.1:8080");

    let main_server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap_fn(|req, srv| {
                HTTP_COUNTER.inc();
                srv.call(req)
            })
            .service(index)
            .service(static_files)
    })
    .shutdown_timeout(settings.graceful_shutdown_timeout_seconds)
    .bind("0.0.0.0:8080")?
    .run();

    let observability_server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .service(metrics)
            .service(health)
            .service(readiness)
    })
    .shutdown_timeout(settings.graceful_shutdown_timeout_seconds)
    .bind("0.0.0.0:8081")?
    .run();

    try_join!(main_server, observability_server)?;
    Ok(())
}
