$ErrorActionPreference = "Stop"
Set-Location (Split-Path -Parent $PSScriptRoot)
New-Item -ItemType Directory -Force -Path bin | Out-Null
cargo build --release
$targetRoot = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path (Get-Location) "target" }
$src = Join-Path $targetRoot "release\dev-team.exe"
if (-not (Test-Path $src)) {
    throw "Built binary not found at $src"
}
try {
    Copy-Item -Force $src "bin\dev-team.exe"
    Write-Host "Installed bin\dev-team.exe"
} catch {
    Write-Warning "Could not overwrite bin\dev-team.exe (is the Dev Team pane still open?). Built binary is at: $src"
}
