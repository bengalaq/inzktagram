//! Benchmark reproducible del pipeline ZK.
//!
//! Mide, para N candidatos y cada algoritmo: ciclos de usuario en la zkVM,
//! tiempo de proving, tiempo de verificación, tamaño del receipt y el
//! baseline nativo (el mismo ranking fuera de la zkVM).
//!
//! Uso:   cargo run --release -p inzktagram-server --bin zkbench
//! Salida: benchmarks/results.csv
//!
//! IMPORTANTE: correr SIN RISC0_DEV_MODE; con dev mode los números no miden nada.

use std::io::Write as _;
use std::time::Instant;

use feed_core::{Candidate, FeedInput, Params, UserConfig};
use inzktagram_server::{prover, prover_dev_mode};

fn synth_candidates(n: u64) -> Vec<Candidate> {
    (0..n)
        .map(|i| {
            let r = (i
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(144_115_188))
                >> 33;
            Candidate {
                post_id: i + 1,
                author_id: (i % 9) + 1,
                created_at: 1_756_000_000 - ((r % 2_880) + 1) * 60,
                likes: (r % 900) as u32,
                comments: (r % 90) as u32,
                length_chars: 80 + ((r % 11) * 60) as u32,
                is_followed: i % 3 != 0,
            }
        })
        .collect()
}

fn make_input(n: u64, alg: u8) -> FeedInput {
    FeedInput {
        config: UserConfig {
            user_id: 99,
            algorithm_id: alg,
            nonce: 1,
        },
        params: Params::default(),
        candidates: synth_candidates(n),
        now: 1_756_000_000,
    }
}

fn sizes_from_env() -> Vec<u64> {
    std::env::var("ZKBENCH_N")
        .ok()
        .map(|s| {
            s.split(',')
                .filter_map(|p| p.trim().parse().ok())
                .collect()
        })
        .filter(|v: &Vec<u64>| !v.is_empty())
        .unwrap_or_else(|| vec![25, 50, 100, 200])
}

fn hardware_line() -> String {
    let cpu = std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("model name"))
                .map(|l| l.split(':').nth(1).unwrap_or("").trim().to_string())
        })
        .unwrap_or_else(|| std::env::consts::ARCH.to_string());
    let cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    format!("{cpu} ({cores} threads), os={}", std::env::consts::OS)
}

fn main() -> anyhow::Result<()> {
    let dev = prover_dev_mode();
    if dev {
        eprintln!("WARNING: RISC0_DEV_MODE is on; numbers are NOT real STARK measurements.");
    }
    println!("zkbench — image ID: {}", prover::image_id_hex());
    let hw = hardware_line();
    println!("hardware: {hw}");

    let sizes = sizes_from_env();
    std::fs::create_dir_all("benchmarks")?;
    let mut csv = std::fs::File::create("benchmarks/results.csv")?;
    writeln!(csv, "# hardware: {hw}")?;
    writeln!(csv, "# image_id: {}", prover::image_id_hex())?;
    writeln!(
        csv,
        "n,algorithm,algorithm_name,user_cycles,proving_ms,verify_us,receipt_kb,native_us,dev_mode"
    )?;

    println!(
        "{:>5} {:>12} {:>12} {:>11} {:>10} {:>11} {:>10}",
        "N", "algorithm", "cycles", "prove(ms)", "verif(us)", "receipt(KB)", "native(us)"
    );

    for n in sizes {
        for alg in 1u8..=3 {
            let input = make_input(n, alg);

            // Baseline nativo: el mismo cómputo fuera de la zkVM.
            let t = Instant::now();
            let _native_feed = feed_core::rank(&input);
            let native_us = t.elapsed().as_micros();

            let res = prover::prove_feed(&input)?;

            let t = Instant::now();
            prover::verify_receipt(&res.receipt)?;
            let verify_us = t.elapsed().as_micros();

            let receipt_kb = bincode::serialize(&res.receipt)?.len() as f64 / 1024.0;
            let name = feed_core::algorithm_name(alg);

            println!(
                "{:>5} {:>12} {:>12} {:>11} {:>10} {:>11.1} {:>10}",
                n, name, res.user_cycles, res.proving_ms, verify_us, receipt_kb, native_us
            );
            writeln!(
                csv,
                "{n},{alg},{name},{},{},{verify_us},{receipt_kb:.1},{native_us},{dev}",
                res.user_cycles, res.proving_ms
            )?;
            csv.flush()?;
        }
    }
    println!("\nResults written to benchmarks/results.csv");
    Ok(())
}
