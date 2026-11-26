param(
    [switch]$Mcp,
    [switch]$Smoke,
    [switch]$Release
)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $RepoRoot

if ($Smoke) {
    cargo run -p proofcraft-app -- --smoke-test
    exit $LASTEXITCODE
}

if ($Mcp) {
    if ($Release -and (Test-Path "$RepoRoot\target\release\proofcraft-mcp.exe")) {
        & "$RepoRoot\target\release\proofcraft-mcp.exe" mcp
    } else {
        cargo run -p proofcraft-mcp -- mcp
    }
    exit $LASTEXITCODE
}

if ($Release -and (Test-Path "$RepoRoot\target\release\proofcraft-app.exe")) {
    & "$RepoRoot\target\release\proofcraft-app.exe"
} else {
    cargo run -p proofcraft-app
}
