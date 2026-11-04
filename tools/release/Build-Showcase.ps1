param([string]$Root = "$PSScriptRoot/../..", [string]$OutputDirectory)
$ErrorActionPreference = 'Stop'
$tool = & "$PSScriptRoot/../native/Build.ps1" -Tool release-showcase
$rootPath = [IO.Path]::GetFullPath($Root)
if (-not $OutputDirectory) { $OutputDirectory = Join-Path $rootPath 'docs/releases/0.2.0' }
$outputPath = [IO.Path]::GetFullPath($OutputDirectory)
& $tool $rootPath $outputPath
if ($LASTEXITCODE -ne 0) { throw 'Release showcase generation failed' }
Get-ChildItem (Join-Path $outputPath '*.png') | Select-Object Name,Length
