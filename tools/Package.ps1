# Build a portable Windows x64 package and enforce the complete release-size budget.
$ErrorActionPreference = 'Stop'
Push-Location "$PSScriptRoot/.."
try {
    $hostTriple = ((& rustc -vV) | Select-String '^host: ').ToString().Substring(6)
    if ($hostTriple -ne 'x86_64-pc-windows-msvc') { throw 'This release script currently supports the Windows x64 MSVC toolchain.' }
    & cargo build --locked --release
    if ($LASTEXITCODE -ne 0) { throw 'Release build failed' }
    $folder = Join-Path (Get-Location) 'dist/Myosotis'
    [void][IO.Directory]::CreateDirectory($folder)
    Copy-Item 'target/release/myosotis.exe' $folder -Force
    Copy-Item 'README.md','LICENSE','THIRD_PARTY_NOTICES.md' $folder -Force
    $installedBytes = (Get-ChildItem $folder -Recurse -File | Measure-Object Length -Sum).Sum
    if ($installedBytes -ge 20000000) { throw "Installed package exceeds 20 MB: $installedBytes bytes" }
    $metadata = (& cargo metadata --no-deps --format-version 1 | ConvertFrom-Json)
    if ($LASTEXITCODE -ne 0) { throw 'Could not read package version' }
    $version = ($metadata.packages | Where-Object name -eq 'myosotis').version
    $archive = Join-Path (Get-Location) "dist/Myosotis-$version-windows-x64.zip"
    Compress-Archive -Path $folder -DestinationPath $archive -Force
    $downloadBytes = (Get-Item $archive).Length
    if ($downloadBytes -ge 10000000) { throw "Download exceeds 10 MB: $downloadBytes bytes" }
    $hasher = [Security.Cryptography.SHA256]::Create()
    $stream = [IO.File]::OpenRead($archive)
    try { $hash = [BitConverter]::ToString($hasher.ComputeHash($stream)).Replace('-','').ToLowerInvariant() }
    finally { $stream.Dispose(); $hasher.Dispose() }
    "$hash  $([IO.Path]::GetFileName($archive))" | Set-Content "$archive.sha256" -Encoding ascii
    Write-Output "Portable package: $archive"
    Write-Output "Downloaded: $downloadBytes bytes; installed: $installedBytes bytes"
    Write-Output "SHA256: $hash"
} finally { Pop-Location }
