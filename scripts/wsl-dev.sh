#!/usr/bin/env bash
# Copia el repo a $HOME/inzktagram (ext4, sin espacios). risc0-build no tolera
# CARGO_TARGET_DIR fuera del workspace ni bien las rutas con espacios de /mnt/c.
# En esta máquina hay que usar Ubuntu 24.04 (glibc 2.39); Ubuntu 20.04 no corre
# el rustc de RISC Zero 3.0.6 (pide GLIBC_2.34).
set -euo pipefail
export PATH="$HOME/.risc0/bin:$HOME/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"
unset CARGO_TARGET_DIR
export RISC0_DEV_MODE="${RISC0_DEV_MODE:-0}"
export RISC0_GUEST_LOGFILE="${RISC0_GUEST_LOGFILE:-$HOME/inzktagram-guest.log}"

SRC="$(cd "$(dirname "$0")/.." && pwd)"
DST="${INZK_BUILD_DIR:-$HOME/inzktagram}"

mkdir -p "$DST"
rsync -a --delete \
  --exclude target --exclude node_modules --exclude .git \
  --exclude '*.sqlite' --exclude '*.sqlite-*' --exclude web/dist \
  "$SRC/" "$DST/"

if [ -d "$SRC/web/dist" ]; then
  rsync -a "$SRC/web/dist/" "$DST/web/dist/"
fi

cd "$DST"
echo "[inzktagram] distro=$(. /etc/os-release; echo $PRETTY_NAME) glibc=$(ldd --version | head -1)"
echo "[inzktagram] build dir=$DST RISC0_DEV_MODE=$RISC0_DEV_MODE"
echo "[inzktagram] cargo=$(command -v cargo) cargo-risczero=$(command -v cargo-risczero) rustc=$(rustc --version)"

case "${1:-run}" in
  test)
    cargo test -p feed-core
    cargo test -p inzktagram-server --test completeness_dev
    ;;
  build)
    cargo build --release -p inzktagram-server --bin server -p verifier-cli
    ;;
  run)
    exec cargo run --release -p inzktagram-server --bin server
    ;;
  *)
    echo "uso: wsl-dev.sh [test|build|run]" >&2
    exit 2
    ;;
esac
