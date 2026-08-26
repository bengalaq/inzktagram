pub mod api;
pub mod db;
pub mod prover;
pub mod seed;

use std::sync::Arc;
use std::time::Duration;

use rusqlite::{params, Connection};
use tokio::sync::Mutex;

pub struct AppState {
    pub db: Mutex<Connection>,
}

pub fn now_epoch() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("reloj antes de 1970")
        .as_secs()
}

/// `true` si el prover está en modo desarrollo (receipts falsos, instantáneos).
pub fn prover_dev_mode() -> bool {
    std::env::var("RISC0_DEV_MODE")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

/// Worker en background: toma vistas de feed pendientes y genera la prueba
/// RISC Zero de cada una. El proving corre en un hilo bloqueante para no
/// frenar el servidor.
pub async fn proof_worker(state: Arc<AppState>) {
    loop {
        let job = {
            let db = state.db.lock().await;
            let row = db.query_row(
                "SELECT id, input_json FROM feed_views WHERE status = 'pending' ORDER BY id LIMIT 1",
                [],
                |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)),
            );
            match row {
                Ok((id, input_json)) => {
                    let _ = db.execute(
                        "UPDATE feed_views SET status = 'proving' WHERE id = ?1",
                        params![id],
                    );
                    Some((id, input_json))
                }
                Err(rusqlite::Error::QueryReturnedNoRows) => None,
                Err(e) => {
                    tracing::error!("worker: error de base de datos: {e}");
                    None
                }
            }
        };

        let Some((view_id, input_json)) = job else {
            tokio::time::sleep(Duration::from_millis(1200)).await;
            continue;
        };

        tracing::info!("worker: generando prueba para la vista {view_id}...");
        let result = tokio::task::spawn_blocking(move || -> anyhow::Result<prover::ProofResult> {
            let input: feed_core::FeedInput = serde_json::from_str(&input_json)?;
            prover::prove_feed(&input)
        })
        .await;

        let flattened: anyhow::Result<prover::ProofResult> = match result {
            Ok(inner) => inner,
            Err(join_err) => Err(anyhow::anyhow!("panic en el prover: {join_err}")),
        };

        let db = state.db.lock().await;
        match flattened.and_then(|res| {
            let bytes = bincode::serialize(&res.receipt)?;
            Ok((res, bytes))
        }) {
            Ok((res, receipt_bytes)) => {
                let journal_json = prover::journal_to_json(&res.journal).to_string();
                let _ = db.execute(
                    "UPDATE feed_views SET status = 'proved', receipt = ?1, journal_json = ?2,
                     proving_ms = ?3, user_cycles = ?4 WHERE id = ?5",
                    params![
                        receipt_bytes,
                        journal_json,
                        res.proving_ms as i64,
                        res.user_cycles as i64,
                        view_id
                    ],
                );
                tracing::info!(
                    "worker: vista {view_id} probada en {} ms ({} ciclos de usuario)",
                    res.proving_ms,
                    res.user_cycles
                );
            }
            Err(e) => {
                tracing::error!("worker: fallo probando la vista {view_id}: {e}");
                let _ = db.execute(
                    "UPDATE feed_views SET status = 'failed', error = ?1 WHERE id = ?2",
                    params![e.to_string(), view_id],
                );
            }
        }
    }
}
