//! Soundness de la *aplicación* (rápido, RISC0_DEV_MODE).
//!
//! No sustituye al test STARK real: acá comprobamos que (1) el guest rechaza
//! candidatos ilegales, (2) los chequeos de algoritmo/hash del journal
//! detectan una mentira. Corre en CI en cada push.

mod common;

use feed_core::{
    claims_match, ALG_ENGAGEMENT, ALG_WELLBEING,
};
use inzktagram_server::prover;

#[test]
fn guest_rejects_unsorted_candidates() {
    std::env::set_var("RISC0_DEV_MODE", "1");
    let mut input = common::sample_input(12, ALG_WELLBEING);
    input.candidates.swap(0, 1);
    assert!(
        prover::prove_feed(&input).is_err(),
        "el guest debió abortar con candidatos desordenados"
    );
}

#[test]
fn guest_rejects_viewer_own_posts() {
    std::env::set_var("RISC0_DEV_MODE", "1");
    let mut input = common::sample_input(12, ALG_WELLBEING);
    input.candidates[0].author_id = input.config.user_id;
    assert!(
        prover::prove_feed(&input).is_err(),
        "el guest debió abortar si el viewer aparece como autor"
    );
}

#[test]
fn claims_reject_wrong_algorithm_and_feed_hash() {
    std::env::set_var("RISC0_DEV_MODE", "1");
    let input = common::sample_input(12, ALG_WELLBEING);
    let res = prover::prove_feed(&input).expect("proving honesto falló");
    prover::verify_receipt(&res.receipt).expect("el receipt honesto debe verificar en dev-mode");

    assert!(
        claims_match(
            &res.journal,
            Some(ALG_WELLBEING),
            Some(&res.journal.feed_hash),
            Some(&res.journal.candidates_hash)
        ),
        "un journal honesto debió coincidir con las expectativas"
    );
    assert!(
        !claims_match(&res.journal, Some(ALG_ENGAGEMENT), None, None),
        "afirmar el algoritmo 1 sobre un journal de bienestar debió fallar"
    );
    let mut wrong_feed = res.journal.feed_hash;
    wrong_feed[0] ^= 0xFF;
    assert!(
        !claims_match(&res.journal, None, Some(&wrong_feed), None),
        "un feed_hash distinto debió ser rechazado"
    );
}
