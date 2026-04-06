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

Invoke-Checked cargo @("fmt", "--all")
