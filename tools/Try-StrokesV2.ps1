param(
    [ValidateSet('Baseline', 'Clean', 'Natural')]
    [string]$Direction = 'Baseline'
)
# Launch a prebuilt local trial. Existing captures and resident apps are left alone.
$ErrorActionPreference = 'Stop'
$root = [IO.Path]::GetFullPath("$PSScriptRoot/..")
$relative = ".tmp/stroke-v2/$($Direction.ToLowerInvariant())-final/myosotis.exe"
if ($Direction -eq 'Baseline') { $relative = '.tmp/stroke-trials/baseline/myosotis.exe' }
$exe = Join-Path $root $relative
if (!(Test-Path -LiteralPath $exe)) {
    throw "Trial executable missing: $exe. See docs/STROKE-INCREMENTAL-RESULTS.md for build instructions."
}
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class MyosotisStrokeTrial {
    [DllImport("user32.dll", CharSet=CharSet.Unicode)]
    public static extern IntPtr FindWindow(string name, IntPtr title);
    [DllImport("user32.dll")]
    public static extern uint GetWindowThreadProcessId(IntPtr window, out uint process);
}
'@
$window = [MyosotisStrokeTrial]::FindWindow('Myosotis.Native.Overlay', [IntPtr]::Zero)
if ($window -ne [IntPtr]::Zero) {
    [uint32]$residentId = 0
    [void][MyosotisStrokeTrial]::GetWindowThreadProcessId($window, [ref]$residentId)
    $resident = Get-Process -Id $residentId -ErrorAction SilentlyContinue
    $location = if ($resident) { $resident.Path } else { "PID $residentId" }
    throw "Myosotis is already running: $location. Finish or cancel any capture, then right-click its tray icon and choose Quit. Rerun this command afterward."
}
if ($Direction -eq 'Baseline') {
    Write-Output 'Baseline: original 0.1.2 drawing behavior, before the stroke experiments. No F6 profile switching.'
} else {
    Write-Output "$Direction incremental experiment: native bitmap caching, mouse taper, stylus pressure. No profile switching or load-dependent fallback."
}
Write-Output 'Right-drag for a white canvas; Ctrl or Enter copies; Esc cancels.'
Start-Process -FilePath $exe -ArgumentList '--capture' -WorkingDirectory $root
