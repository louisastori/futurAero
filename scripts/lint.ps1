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

Invoke-Checked cargo @("fmt", "--all", "--check")
Invoke-Checked cargo @("clippy", "--workspace", "--all-targets", "--", "-D", "warnings")
Invoke-Checked cargo @("fmt", "--manifest-path", "apps/desktop/src-tauri/Cargo.toml", "--check")
Invoke-Checked cargo @("clippy", "--manifest-path", "apps/desktop/src-tauri/Cargo.toml", "--all-targets", "--", "-D", "warnings")
Invoke-Checked npm @("run", "lint")
