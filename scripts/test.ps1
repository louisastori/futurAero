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

Invoke-Checked cargo @("test", "--workspace")
Invoke-Checked cargo @("test", "--manifest-path", "apps/desktop/src-tauri/Cargo.toml")
Invoke-Checked npm @("test")
