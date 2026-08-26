use std::sync::Arc;

use axum::Router;
use inzktagram_server::{api, db, now_epoch, proof_worker, prover, prover_dev_mode, seed, AppState};
use tokio::sync::Mutex;
use tower_http::services::{ServeDir, ServeFile};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    let db_path = std::env::var("INZK_DB").unwrap_or_else(|_| "inzktagram.sqlite".to_string());
    let conn = db::open(&db_path)?;
    seed::seed_if_empty(&conn, now_epoch())?;

    let state = Arc::new(AppState { db: Mutex::new(conn) });
    tokio::spawn(proof_worker(state.clone()));

    let dist = std::env::var("INZK_WEB_DIST").unwrap_or_else(|_| "web/dist".to_string());
    let static_files =
        ServeDir::new(&dist).not_found_service(ServeFile::new(format!("{dist}/index.html")));

    let app = Router::new()
        .nest("/api", api::router())
        .fallback_service(static_files)
        .with_state(state);

    if prover_dev_mode() {
        tracing::warn!("RISC0_DEV_MODE activo: los receipts son FALSOS (solo desarrollo)");
    }
    tracing::info!("image ID del guest: {}", prover::image_id_hex());
    tracing::info!("inZKtagram escuchando en http://localhost:8080");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
