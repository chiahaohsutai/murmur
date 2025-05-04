use actix_files as fs;
use actix_web::{App, HttpServer, web};
use dotenv::dotenv;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod audio;
mod routes;
mod state;

use audio::create_stt_model;
use routes::{index, ws};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing::info!("Loading enviorment variables.");
    dotenv().ok();
    whisper_rs::install_logging_hooks();

    let filter = EnvFilter::new("Murmur=trace,Murmur::routes=trace");
    tracing_subscriber::registry()
        .with(filter)
        .with(
            tracing_subscriber::fmt::layer()
                .pretty()
                .with_ansi(true)
                .with_target(false)
                .with_file(false)
                .compact(),
        )
        .init();

    let port = 8080;
    let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let socket = SocketAddr::new(ip, port);

    tracing::info!("Creating Whisper context.");
    let ctx = match std::env::var("STT_MODEL_PATH") {
        Ok(path) => match create_stt_model(path) {
            Ok(context) => context,
            Err(err) => {
                tracing::error!("Failed to load model into memory: {}", err);
                std::process::exit(1);
            }
        },
        Err(err) => {
            tracing::error!("Missing STT_MODEL_PATH environment variable: {}", err);
            std::process::exit(1);
        }
    };
    let whisper = Arc::new(ctx);

    tracing::info!("Starting server at http://127.0.0.1:8080.");
    let server = HttpServer::new(move || {
        App::new()
            .wrap(tracing_actix_web::TracingLogger::default())
            .app_data(web::Data::new(state::AppState::new(Arc::clone(&whisper))))
            .service(
                fs::Files::new("/assets", "./assets")
                    .prefer_utf8(true)
                    .disable_content_disposition()
                    .show_files_listing(),
            )
            .service(index)
            .service(ws)
    })
    .keep_alive(Duration::from_secs(60));
    server.bind(socket)?.run().await
}
