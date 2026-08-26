#!/usr/bin/env bash
# Entorno común. Preferí scripts/wsl-dev.sh (copia a ~/inzktagram).
# RISC Zero 3.0.6 necesita glibc >= 2.34 (Ubuntu 22.04+ / 24.04).
export PATH="$HOME/.risc0/bin:$HOME/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"
unset CARGO_TARGET_DIR
cd "$(dirname "${BASH_SOURCE[0]}")/.."
