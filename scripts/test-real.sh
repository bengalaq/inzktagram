#!/usr/bin/env bash
# Completeness y soundness con pruebas STARK reales (lento: minutos de CPU).
set -euo pipefail
source "$(dirname "$0")/env.sh"
cargo test --release -p inzktagram-server --test real_proofs -- --ignored --nocapture
