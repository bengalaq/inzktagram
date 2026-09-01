#!/usr/bin/env bash
# Un comando: levanta inZKtagram con Docker (Windows/macOS/Linux).
set -euo pipefail
cd "$(dirname "$0")"

if ! command -v docker >/dev/null 2>&1; then
  echo "No se encontró Docker. Instalá Docker Desktop (o docker + compose) y reintentá." >&2
  exit 1
fi

if ! docker info >/dev/null 2>&1; then
  echo "Docker está instalado pero el motor no responde. Abrí Docker Desktop y esperá a que arranque." >&2
  exit 1
fi

if docker compose version >/dev/null 2>&1; then
  COMPOSE=(docker compose)
elif command -v docker-compose >/dev/null 2>&1; then
  COMPOSE=(docker-compose)
else
  echo "Falta el plugin Compose (docker compose). Actualizá Docker Desktop." >&2
  exit 1
fi

echo "inZKtagram → http://localhost:8080"
echo "La primera vez baja la toolchain RISC Zero y puede tardar varios minutos."
echo "Ctrl+C detiene el contenedor."
echo
exec "${COMPOSE[@]}" up --build
