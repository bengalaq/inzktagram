#!/usr/bin/env bash
# Verifica un receipt de inZKtagram. Los .receipt van en download_receipts/.
#
#   ./verify.sh inzktagram_view_15.receipt --expect-algorithm 2 --expect-feed-hash <hash>
#   ./verify.sh 15 --expect-algorithm 2 --expect-feed-hash <hash>
set -euo pipefail
cd "$(dirname "$0")"
mkdir -p download_receipts

if [[ $# -lt 1 ]]; then
  echo "Uso: ./verify.sh <archivo.receipt|view_id> [--expect-algorithm N] [--expect-feed-hash HASH]" >&2
  echo >&2
  echo "Guardá el receipt descargado en: $(pwd)/download_receipts" >&2
  echo "Ejemplo:" >&2
  echo "  ./verify.sh inzktagram_view_15.receipt --expect-algorithm 2 --expect-feed-hash abc..." >&2
  exit 1
fi

first="$1"
shift
if [[ "$first" =~ ^[0-9]+$ ]]; then
  name="inzktagram_view_${first}.receipt"
else
  name="$(basename "$first")"
fi

host_path="download_receipts/$name"
if [[ ! -f "$host_path" ]]; then
  echo "No encuentro $host_path" >&2
  echo "Descargá el receipt y copialo a download_receipts/" >&2
  exit 1
fi

if ! command -v docker >/dev/null 2>&1; then
  echo "No se encontró Docker." >&2
  exit 1
fi

if docker compose version >/dev/null 2>&1; then
  COMPOSE=(docker compose)
elif command -v docker-compose >/dev/null 2>&1; then
  COMPOSE=(docker-compose)
else
  echo "Falta docker compose." >&2
  exit 1
fi

if [[ -n "$("${COMPOSE[@]}" ps --status running -q inzktagram 2>/dev/null || true)" ]]; then
  exec "${COMPOSE[@]}" exec -T inzktagram verifier-cli "/receipts/$name" "$@"
fi
exec "${COMPOSE[@]}" run --rm --no-deps inzktagram verifier-cli "/receipts/$name" "$@"
