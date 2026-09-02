#!/usr/bin/env bash
# Real STARK benchmarks. Writes benchmarks/results.csv
#   ./scripts/bench.sh
#   ZKBENCH_N=25,50 ./scripts/bench.sh
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p benchmarks
if docker compose version >/dev/null 2>&1; then
  COMPOSE=(docker compose)
else
  COMPOSE=(docker-compose)
fi
echo "inZKtagram zkbench → benchmarks/results.csv (this takes a while)"
exec "${COMPOSE[@]}" --profile bench run --rm zkbench
