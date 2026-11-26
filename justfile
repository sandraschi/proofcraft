set windows-shell := ["powershell.exe", "-NoProfile", "-Command"]

default:
    @just --list

check:
    $env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"; cargo clippy --workspace --all-targets --all-features -- -D warnings

test:
    $env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"; cargo test --workspace

fmt-check:
    $env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"; cargo fmt --all --check

run-mcp:
    $env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"; cargo run -p proofcraft-mcp -- mcp

run-app:
    $env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"; cargo run -p proofcraft-app

smoke:
    $env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"; cargo run -p proofcraft-app -- --smoke-test

build-release:
    $env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"; cargo build --release --workspace

ci: check test fmt-check
