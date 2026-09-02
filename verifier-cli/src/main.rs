//! Independent verifier for inZKtagram receipts.
//!
//!   verifier-cli <receipt> --expect-algorithm 2 --expect-feed-hash <hex>
//!   verifier-cli --candidates dump.json          # print candidates_hash
//!   verifier-cli <receipt> --candidates dump.json  # also check the journal hash
//!
//! Default image ID is the guest compiled with this binary. Rebuilding the
//! guest from source is how an auditor checks they are verifying the right
//! program.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use feed_core::{
    algorithm_name, assemble_candidates, claims_match, hash_candidates, Candidate, FollowEdge,
    Journal, PublicPost,
};
use risc0_zkvm::{sha::Digest, Receipt};
use serde::Deserialize;

#[derive(Parser)]
#[command(
    name = "verifier-cli",
    about = "Verify RISC Zero receipts of inZKtagram feeds"
)]
struct Args {
    /// Downloaded .receipt file. Optional if you only want to hash --candidates.
    receipt: Option<PathBuf>,

    /// Expected guest image ID, hex (default: the ID baked into this binary)
    #[arg(long)]
    image_id: Option<String>,

    /// Algorithm the user chose: 1=Engagement, 2=Wellbeing, 3=Mixed
    #[arg(long)]
    expect_algorithm: Option<u8>,

    /// SHA-256 hex of the feed the UI actually rendered
    #[arg(long)]
    expect_feed_hash: Option<String>,

    /// SHA-256 hex of the candidate set (if you hashed it separately)
    #[arg(long)]
    expect_candidates_hash: Option<String>,

    /// JSON dump: either `[Candidate, ...]` or `{user_id, posts, follows}`.
    /// GET /api/audit/{user_id} returns the second shape.
    #[arg(long)]
    candidates: Option<PathBuf>,
}

#[derive(Deserialize)]
struct AuditDump {
    user_id: u64,
    posts: Vec<PublicPost>,
    follows: Vec<FollowEdge>,
}

fn parse_digest(hex_str: &str) -> anyhow::Result<Digest> {
    let bytes = hex::decode(hex_str.trim())?;
    let arr: [u8; 32] = bytes
        .as_slice()
        .try_into()
        .map_err(|_| anyhow::anyhow!("digest must be 32 bytes"))?;
    Ok(Digest::from(arr))
}

fn parse_hash32(hex_str: &str) -> anyhow::Result<[u8; 32]> {
    let bytes = hex::decode(hex_str.trim())?;
    bytes
        .as_slice()
        .try_into()
        .map_err(|_| anyhow::anyhow!("hash must be 32 bytes"))
}

fn load_candidates_hash(path: &PathBuf) -> anyhow::Result<[u8; 32]> {
    let raw = std::fs::read_to_string(path)?;
    let val: serde_json::Value = serde_json::from_str(&raw)?;
    let cands: Vec<Candidate> = if val.get("posts").is_some() {
        let dump: AuditDump = serde_json::from_value(val)?;
        assemble_candidates(dump.user_id, &dump.posts, &dump.follows)
    } else {
        serde_json::from_value(val)?
    };
    Ok(hash_candidates(&cands))
}

fn run(args: &Args) -> anyhow::Result<bool> {
    if args.receipt.is_none() && args.candidates.is_none() {
        anyhow::bail!("pass a receipt and/or --candidates <file.json>");
    }

    let candidates_hash = match &args.candidates {
        Some(path) => Some(load_candidates_hash(path)?),
        None => None,
    };

    if args.receipt.is_none() {
        let h = candidates_hash.expect("checked above");
        println!("candidates_hash   : {}", hex::encode(h));
        println!("(no receipt: hash only; this is not a STARK check)");
        return Ok(true);
    }

    let receipt_path = args.receipt.as_ref().unwrap();
    let bytes = std::fs::read(receipt_path)?;
    let receipt: Receipt = bincode::deserialize(&bytes)?;

    let image_id = match &args.image_id {
        Some(s) => parse_digest(s)?,
        None => Digest::from(inzktagram_methods::FEED_GUEST_ID),
    };
    println!("Expected image ID : {image_id}");

    match receipt.verify(image_id) {
        Ok(()) => println!("STARK proof       : VALID ✓"),
        Err(e) => {
            println!("STARK proof       : INVALID ✗ ({e})");
            return Ok(false);
        }
    }

    let journal: Journal = receipt.journal.decode()?;
    println!("\n--- Journal (public statement) ---");
    println!(
        "Proved algorithm  : {} ({})",
        journal.algorithm_id,
        algorithm_name(journal.algorithm_id)
    );
    println!("Config hash       : {}", hex::encode(journal.config_hash));
    println!("Params hash       : {}", hex::encode(journal.params_hash));
    println!("Candidates hash   : {}", hex::encode(journal.candidates_hash));
    println!("Feed hash         : {}", hex::encode(journal.feed_hash));
    println!("Timestamp         : {}", journal.timestamp);

    let expect_feed = args
        .expect_feed_hash
        .as_deref()
        .map(parse_hash32)
        .transpose()?;
    let expect_cands_flag = args
        .expect_candidates_hash
        .as_deref()
        .map(parse_hash32)
        .transpose()?;
    let expect_cands = expect_cands_flag.or(candidates_hash);

    println!("\n--- Application checks ---");
    let mut all_ok = true;

    if let Some(expected) = args.expect_algorithm {
        let ok = journal.algorithm_id == expected;
        all_ok &= ok;
        println!(
            "Chosen algorithm ({} - {}) : {}",
            expected,
            algorithm_name(expected),
            if ok { "MATCH ✓" } else { "MISMATCH ✗" }
        );
    }
    if let Some(expected) = &expect_feed {
        let ok = &journal.feed_hash == expected;
        all_ok &= ok;
        println!(
            "Displayed feed              : {}",
            if ok {
                "MATCH ✓"
            } else {
                "MISMATCH ✗ (the server showed a different feed than it proved)"
            }
        );
    }
    if let Some(expected) = &expect_cands {
        let ok = &journal.candidates_hash == expected;
        all_ok &= ok;
        println!(
            "Candidate set               : {}",
            if ok {
                "MATCH ✓"
            } else {
                "MISMATCH ✗ (the proved input set ≠ the public dump you hashed)"
            }
        );
    }

    if !claims_match(
        &journal,
        args.expect_algorithm,
        expect_feed.as_ref(),
        expect_cands.as_ref(),
    ) {
        all_ok = false;
    }

    Ok(all_ok)
}

fn main() -> ExitCode {
    let args = Args::parse();
    match run(&args) {
        Ok(true) => {
            println!("\nRESULT: verification SUCCEEDED");
            ExitCode::SUCCESS
        }
        Ok(false) => {
            println!("\nRESULT: verification FAILED");
            ExitCode::FAILURE
        }
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
