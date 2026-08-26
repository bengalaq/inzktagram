//! Completeness (rápido, con RISC0_DEV_MODE): el guest ejecuta exactamente el
//! mismo ranking que el host y produce un journal consistente que el
//! verificador acepta. Corre en CI en segundos.
//!
//! Nota: este binario de test fija RISC0_DEV_MODE para sí mismo; las pruebas
//! REALES (STARK) están en `real_proofs.rs`, marcadas #[ignore].

mod common;

use feed_core::{ALG_ENGAGEMENT, ALG_MIXED, ALG_WELLBEING};
use inzktagram_server::prover;

#[test]
fn completeness_journal_matches_native_execution() {
    std::env::set_var("RISC0_DEV_MODE", "1");
    for alg in [ALG_ENGAGEMENT, ALG_WELLBEING, ALG_MIXED] {
        let input = common::sample_input(30, alg);
        let res = prover::prove_feed(&input).expect("proving falló");
        // En dev mode verify() acepta el receipt falso: sirve para validar el
        // pipeline y la consistencia guest/host, no la solidez criptográfica.
        prover::verify_receipt(&res.receipt).expect("verificación falló");

        let native = feed_core::make_journal(&input, &feed_core::rank(&input));
        assert_eq!(
            res.journal, native,
            "el guest produjo un journal distinto al cómputo nativo (alg {alg})"
        );
    }
}
