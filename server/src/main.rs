use std::path::PathBuf;

use app::App;
use axum_server::tls_rustls::RustlsConfig;
use tracing::info;
use tracing_subscriber::prelude::*;

use crate::error::Result;

mod app;
mod error;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .json()
                .with_current_span(false)
                .with_span_list(true),
        )
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                let target = "server";
                tracing_subscriber::EnvFilter::new(format!("{target}=trace"))
            }),
        )
        .init();

    let name = "localhost";
    let port = 9999;

    let bind = format!("localhost:{port}");
    info!(bind, "binding server");

    let app = App::new(name, &bind)?;

    let bind = format!("127.0.0.1:{port}");

    let config = RustlsConfig::from_pem_file(
        PathBuf::from("certs/cert.pem"),
        PathBuf::from("certs/key.pem"),
    )
    .await?;

    axum_server::bind_rustls(bind.parse()?, config)
        .serve(app.into_router()?.into_make_service())
        .await?;

    Ok(())
}
