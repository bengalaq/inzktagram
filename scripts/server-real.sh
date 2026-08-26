#!/usr/bin/env bash
set -euo pipefail
export RISC0_DEV_MODE=0
exec "$(dirname "$0")/wsl-dev.sh" run
