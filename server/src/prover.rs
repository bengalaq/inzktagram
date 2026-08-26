//! Lado host de RISC Zero: generación y verificación de receipts.

use anyhow::Result;
use feed_core::{FeedInput, Journal};
use inzktagram_methods::{FEED_GUEST_ELF, FEED_GUEST_ID};
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};

pub struct ProofResult {
    pub receipt: Receipt,
    pub journal: Journal,
    pub proving_ms: u64,
    pub user_cycles: u64,
}

/// Ejecuta el guest sobre `input` y devuelve el receipt (la prueba ZK).
/// Con RISC0_DEV_MODE=1 devuelve un receipt falso instantáneo (solo desarrollo).
pub fn prove_feed(input: &FeedInput) -> Result<ProofResult> {
    let env = ExecutorEnv::builder().write(input)?.build()?;
    let start = std::time::Instant::now();
    let info = default_prover().prove(env, FEED_GUEST_ELF)?;
    let proving_ms = start.elapsed().as_millis() as u64;
    let journal: Journal = info.receipt.journal.decode()?;
    Ok(ProofResult {
        receipt: info.receipt,
        journal,
        proving_ms,
        user_cycles: info.stats.user_cycles,
    })
}

/// Verifica el receipt contra el image ID del guest y devuelve su journal.
pub fn verify_receipt(receipt: &Receipt) -> Result<Journal> {
    receipt.verify(FEED_GUEST_ID)?;
    Ok(receipt.journal.decode()?)
}

/// Image ID del guest en hexadecimal (identifica unívocamente el programa).
pub fn image_id_hex() -> String {
    risc0_zkvm::sha::Digest::from(FEED_GUEST_ID).to_string()
}

/// Serializa el journal a JSON con los hashes en hexadecimal.
pub fn journal_to_json(j: &Journal) -> serde_json::Value {
    serde_json::json!({
        "algorithm_id": j.algorithm_id,
        "algorithm_name": feed_core::algorithm_name(j.algorithm_id),
        "config_hash": hex::encode(j.config_hash),
        "params_hash": hex::encode(j.params_hash),
        "candidates_hash": hex::encode(j.candidates_hash),
        "feed_hash": hex::encode(j.feed_hash),
        "timestamp": j.timestamp,
    })
}
