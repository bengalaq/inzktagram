# Un comando: levanta inZKtagram con Docker (Windows / PowerShell).
Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot

if (-not (Get-Command docker -ErrorAction SilentlyContinue)) {
    Write-Error "No se encontró Docker. Instalá Docker Desktop y reintentá."
    exit 1
}
try {
    docker info | Out-Null
} catch {
    Write-Error "Docker está instalado pero el motor no responde. Abrí Docker Desktop y esperá a que arranque."
    exit 1
}

Write-Host "inZKtagram → http://localhost:8080"
Write-Host "La primera vez baja la toolchain RISC Zero y puede tardar varios minutos."
Write-Host "Ctrl+C detiene el contenedor."
Write-Host ""

docker compose version | Out-Null
if ($LASTEXITCODE -eq 0) {
    docker compose up --build
} else {
    docker-compose up --build
}
