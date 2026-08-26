# Lanza inZKtagram en WSL Ubuntu 24.04 (RISC Zero 3.0.6 necesita glibc >= 2.34).
param(
    [ValidateSet("test", "build", "run")]
    [string]$Command = "run"
)
$root = Split-Path -Parent $PSScriptRoot
wsl -d Ubuntu-24.04 --cd $root -- bash --noprofile --norc scripts/wsl-dev.sh $Command
