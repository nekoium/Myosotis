param([Parameter(Mandatory=$true)][ValidateSet('stroke-probe','release-showcase')][string]$Tool)
$ErrorActionPreference = 'Stop'
$target = [IO.Path]::GetFullPath("$PSScriptRoot/../../target/native-tools")
& cargo build --locked --release --manifest-path "$PSScriptRoot/Cargo.toml" --target-dir $target --bin $Tool
if ($LASTEXITCODE -ne 0) { throw "Failed to build $Tool" }
Join-Path $target "release/$Tool.exe"
