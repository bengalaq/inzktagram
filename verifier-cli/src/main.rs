//! Verificador independiente de inZKtagram.
//!
//! Permite a un usuario (o auditor) verificar SIN CONFIAR EN EL SERVIDOR que
//! el feed que recibió fue calculado con el algoritmo que eligió:
//!
//!   verifier-cli inzktagram_view_42.receipt --expect-algorithm 2 \
//!       --expect-feed-hash <hex del feed mostrado>
//!
//! El image ID por defecto es el del guest compilado junto a este binario:
//! cualquiera puede recompilar el guest desde el código fuente y comprobar
//! que el ID coincide.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use feed_core::{algorithm_name, Journal};
use risc0_zkvm::{sha::Digest, Receipt};

#[derive(Parser)]
#[command(
    name = "verifier-cli",
    about = "Verifica receipts RISC Zero de feeds de inZKtagram"
)]
struct Args {
    /// Archivo .receipt descargado de la plataforma
    receipt: PathBuf,

    /// Image ID esperado del guest, en hex (por defecto: el embebido en este binario)
    #[arg(long)]
    image_id: Option<String>,

    /// Algoritmo que el usuario eligió: 1=Engagement, 2=Bienestar, 3=Mixto
    #[arg(long)]
    expect_algorithm: Option<u8>,

    /// Hash SHA-256 (hex) del feed que la plataforma mostró
    #[arg(long)]
    expect_feed_hash: Option<String>,
}

fn parse_digest(hex_str: &str) -> anyhow::Result<Digest> {
    let bytes = hex::decode(hex_str.trim())?;
    let arr: [u8; 32] = bytes
        .as_slice()
        .try_into()
        .map_err(|_| anyhow::anyhow!("el digest debe tener 32 bytes"))?;
    Ok(Digest::from(arr))
}

fn run(args: &Args) -> anyhow::Result<bool> {
    let bytes = std::fs::read(&args.receipt)?;
    let receipt: Receipt = bincode::deserialize(&bytes)?;

    let image_id = match &args.image_id {
        Some(s) => parse_digest(s)?,
        None => Digest::from(inzktagram_methods::FEED_GUEST_ID),
    };
    println!("Image ID esperado : {image_id}");

    let mut all_ok = true;

    match receipt.verify(image_id) {
        Ok(()) => println!("Prueba STARK      : VÁLIDA ✓"),
        Err(e) => {
            println!("Prueba STARK      : INVÁLIDA ✗ ({e})");
            return Ok(false);
        }
    }

    let journal: Journal = receipt.journal.decode()?;
    println!("\n--- Journal (parte pública de la prueba) ---");
    println!(
        "Algoritmo probado : {} ({})",
        journal.algorithm_id,
        algorithm_name(journal.algorithm_id)
    );
    println!("Hash config       : {}", hex::encode(journal.config_hash));
    println!("Hash parámetros   : {}", hex::encode(journal.params_hash));
    println!("Hash candidatos   : {}", hex::encode(journal.candidates_hash));
    println!("Hash del feed     : {}", hex::encode(journal.feed_hash));
    println!("Timestamp         : {}", journal.timestamp);

    println!("\n--- Chequeos ---");
    if let Some(expected) = args.expect_algorithm {
        let ok = journal.algorithm_id == expected;
        all_ok &= ok;
        println!(
            "Algoritmo elegido ({} - {}) : {}",
            expected,
            algorithm_name(expected),
            if ok { "COINCIDE ✓" } else { "NO COINCIDE ✗" }
        );
    }
    if let Some(expected_hex) = &args.expect_feed_hash {
        let ok = hex::encode(journal.feed_hash) == expected_hex.trim().to_lowercase();
        all_ok &= ok;
        println!(
            "Feed mostrado                : {}",
            if ok { "COINCIDE ✓" } else { "NO COINCIDE ✗ (el servidor mostró un feed distinto al probado)" }
        );
    }

    Ok(all_ok)
}

fn main() -> ExitCode {
    let args = Args::parse();
    match run(&args) {
        Ok(true) => {
            println!("\nRESULTADO: verificación EXITOSA");
            ExitCode::SUCCESS
        }
        Ok(false) => {
            println!("\nRESULTADO: verificación FALLIDA");
            ExitCode::FAILURE
        }
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
