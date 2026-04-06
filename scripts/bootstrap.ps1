$ErrorActionPreference = "Stop"

Write-Host "FutureAero bootstrap scaffold"
Write-Host "Rust toolchain:"
cargo --version
Write-Host "Node:"
node --version
Write-Host "If pnpm is not available locally, run: corepack enable"

