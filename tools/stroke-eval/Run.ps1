param(
    [ValidateSet('Baseline','Clean','Natural')][string]$Direction = 'Baseline',
    [int[]]$Densities = @(0,5,25),
    [int]$Frames = 12,
    [int]$Repeat = 1,
    [string]$Executable,
    [string]$ResultName,
    [ValidateRange(1,60)][int]$Seconds = 2,
    [switch]$Interactive
)
# Diagnostic spike: drives only a process it starts. Modes -2/-4 test copying; use Check-Functions.ps1 to preserve clipboard formats.
$ErrorActionPreference = 'Stop'
if (!$Interactive) { throw 'Native replay requires exclusive cursor and focus control for the entire run. For cursor-free stress checks use cargo test --locked --release offscreen_stroke_stress -- --ignored --nocapture. Pass -Interactive only when the desktop is available.' }
$root = [IO.Path]::GetFullPath("$PSScriptRoot/../..")
$tool = & "$PSScriptRoot/../native/Build.ps1" -Tool stroke-probe
$exe = Join-Path $root ".tmp/stroke-trials/$($Direction.ToLowerInvariant())/myosotis.exe"
if ($Executable) { $exe = [IO.Path]::GetFullPath($Executable) }
if (-not $ResultName) { $ResultName = $Direction.ToLowerInvariant() }
if ($ResultName -notmatch '^[a-zA-Z0-9_-]+$') { throw 'ResultName must be a simple folder name' }
$folder = Join-Path $root ".tmp/stroke-eval/results/$ResultName-$Repeat"
if (Test-Path $folder) { throw "Results already exist: $folder" }
[IO.Directory]::CreateDirectory($folder) | Out-Null
& $tool --interactive $exe $folder ($Densities -join ',') $Frames $Seconds
if ($LASTEXITCODE -ne 0) { throw "Stroke probe failed; results in $folder are incomplete or invalid" }
