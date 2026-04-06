$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $true

function Invoke-Checked {
  param(
    [Parameter(Mandatory = $true)]
    [string]$FilePath,
    [Parameter(Mandatory = $true)]
    [string[]]$Arguments
  )

  & $FilePath @Arguments
  if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
  }
}

$coverageGate = Get-Content "config/coverage-gate.json" | ConvertFrom-Json
$rustCoverage = $coverageGate.rust

Invoke-Checked cargo @("test", "--workspace")
Invoke-Checked cargo @(
  "llvm-cov",
  "--workspace",
  "--all-features",
  "--lcov",
  "--output-path",
  "coverage.lcov",
  "--fail-under-lines",
  "$($rustCoverage.lines)",
  "--fail-under-functions",
  "$($rustCoverage.functions)",
  "--fail-under-regions",
  "$($rustCoverage.regions)"
)
Invoke-Checked cargo @("test", "--manifest-path", "apps/desktop/src-tauri/Cargo.toml")
Invoke-Checked npm @("test")
