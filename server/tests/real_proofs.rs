//! Completeness y soundness con pruebas STARK REALES (lentas, minutos de CPU).
//!
//! Correr con:
//!   cargo test --release -p inzktagram-server --test real_proofs -- --ignored

mod common;

use feed_core::ALG_WELLBEING;
use inzktagram_server::prover;
use risc0_zkvm::sha::Digest;

#[test]
#[ignore = "prueba STARK real: lenta. Correr con -- --ignored (sin RISC0_DEV_MODE)"]
fn completeness_and_soundness_real() {
    std::env::remove_var("RISC0_DEV_MODE");
    let input = common::sample_input(15, ALG_WELLBEING);
    let res = prover::prove_feed(&input).expect("proving real falló");

    // --- Completeness: un cómputo honesto produce una prueba que verifica. ---
    let journal = prover::verify_receipt(&res.receipt).expect("una prueba válida fue rechazada");
    let native = feed_core::make_journal(&input, &feed_core::rank(&input));
    assert_eq!(journal, native);

    // --- Soundness 1: adulterar el journal (p. ej. afirmar que se usó otro
    // algoritmo) debe invalidar la prueba. ---
    let mut tampered = res.receipt.clone();
    tampered.journal.bytes[0] ^= 0xFF;
    assert!(
        tampered.verify(inzktagram_methods::FEED_GUEST_ID).is_err(),
        "un journal adulterado fue aceptado"
    );

    // --- Soundness 2: la prueba no debe verificar contra otro programa
    // (image ID distinto). ---
    let wrong_id = Digest::from([7u32; 8]);
    assert!(
        res.receipt.verify(wrong_id).is_err(),
        "la prueba verificó contra un image ID incorrecto"
    );
}
