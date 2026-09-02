# Verifica un receipt de inZKtagram con el verifier-cli de la imagen Docker.
# Los .receipt tienen que estar en download_receipts\ (esta carpeta del proyecto).
#
#   .\verify.ps1 inzktagram_view_15.receipt --expect-algorithm 2 --expect-feed-hash <hash>
#   .\verify.ps1 15 --expect-algorithm 2 --expect-feed-hash <hash>
Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot

$dir = Join-Path $PSScriptRoot "download_receipts"
if (-not (Test-Path $dir)) {
    New-Item -ItemType Directory -Path $dir | Out-Null
}

if ($args.Count -lt 1) {
    Write-Host "Uso: .\verify.ps1 <archivo.receipt|view_id> [--expect-algorithm N] [--expect-feed-hash HASH] [--candidates dump.json]"
    Write-Host ""
    Write-Host "Guardá el receipt descargado en:"
    Write-Host "  $dir"
    Write-Host ""
    Write-Host "Ejemplo:"
    Write-Host "  .\verify.ps1 inzktagram_view_15.receipt --expect-algorithm 2 --expect-feed-hash abc..."
    exit 1
}

$first = [string]$args[0]
$rest = @()
if ($args.Count -gt 1) { $rest = $args[1..($args.Count - 1)] }

if ($first -match '^\d+$') {
    $name = "inzktagram_view_$first.receipt"
} else {
    $name = [IO.Path]::GetFileName($first)
}

$hostPath = Join-Path $dir $name
if (-not (Test-Path $hostPath)) {
    Write-Error "No encuentro $hostPath`nDescargá el receipt y copialo a download_receipts\"
}

if (-not (Get-Command docker -ErrorAction SilentlyContinue)) {
    Write-Error "No se encontró Docker. Instalá Docker Desktop y reintentá."
}

$cli = @("verifier-cli", "/receipts/$name") + $rest

$running = docker compose ps --status running -q inzktagram 2>$null
if ($running) {
    docker compose exec -T inzktagram @cli
} else {
    docker compose run --rm --no-deps inzktagram @cli
}
exit $LASTEXITCODE
