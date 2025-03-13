use crate::config::Settings;
use crate::{health, metrics, static_assets};
use actix_web::dev::{Server, Service};
use actix_web::middleware::Logger;
use actix_web::{App, HttpServer};
use std::io;

pub(crate) async fn start_main_server(settings: &Settings) -> Result<Server, io::Error> {
    Ok(HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap_fn(|req, srv| {
                metrics::HTTP_COUNTER.inc();
                srv.call(req)
            })
            .configure(health::routes)
            .configure(static_assets::routes)
    })
    .shutdown_timeout(settings.graceful_shutdown_timeout_seconds)
    .bind(format!("0.0.0.0:{}", settings.port))?
    .run())
}
