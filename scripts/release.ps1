param(
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $RepoRoot

if (-not $SkipBuild) {
    cargo fmt --all --check
    if ($LASTEXITCODE -ne 0) { throw "cargo fmt check failed" }
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    if ($LASTEXITCODE -ne 0) { throw "cargo clippy failed" }
    cargo test --workspace
    if ($LASTEXITCODE -ne 0) { throw "cargo test failed" }
    cargo build --release --workspace
    if ($LASTEXITCODE -ne 0) { throw "cargo release build failed" }
}

$Dist = Join-Path $RepoRoot "dist"
if (Test-Path $Dist) { Remove-Item -Recurse -Force $Dist }
New-Item -ItemType Directory -Path $Dist | Out-Null

$AppExe = Join-Path $RepoRoot "target\release\proofcraft-app.exe"
$McpExe = Join-Path $RepoRoot "target\release\proofcraft-mcp.exe"

if (-not (Test-Path $AppExe)) { throw "Missing $AppExe" }
if (-not (Test-Path $McpExe)) { throw "Missing $McpExe" }

Copy-Item $AppExe (Join-Path $Dist "proofcraft-app.exe")
Copy-Item $McpExe (Join-Path $Dist "proofcraft-mcp.exe")

$ZipPath = Join-Path $Dist "proofcraft-portable.zip"
Compress-Archive -Path (Join-Path $Dist "proofcraft-app.exe"), (Join-Path $Dist "proofcraft-mcp.exe"), (Join-Path $RepoRoot "README.md"), (Join-Path $RepoRoot "HELP.md") -DestinationPath $ZipPath

$Sums = Get-ChildItem $Dist -File | ForEach-Object {
    $h = (Get-FileHash $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
    "$h  $($_.Name)"
}
Set-Content -Path (Join-Path $Dist "SHA256SUMS.txt") -Value $Sums -Encoding UTF8

Write-Host "Staged release artifacts in ${Dist}:"
Get-ChildItem $Dist | Select-Object Name, Length
