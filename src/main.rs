use actix_web::dev::Server;
use futures::try_join;

use std::env;
mod config;
mod health;
mod metrics;
mod server;
mod static_assets;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let settings = config::load_config().unwrap_or_default();
    if env::var("RUST_LOG").is_err() {
        unsafe {
            env::set_var("RUST_LOG", settings.logger.level.clone());
        }
    }

    env_logger::init();
    let main_server: Server = server::start_main_server(&settings).await?;
    let metrics_server = metrics::start_metrics_server(&settings)?;
    try_join!(main_server, metrics_server)?;
    Ok(())
}
