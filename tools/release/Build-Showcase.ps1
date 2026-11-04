param([string]$Root = "$PSScriptRoot/../..")
$ErrorActionPreference='Stop'
Add-Type -AssemblyName System.Drawing,System.Windows.Forms.DataVisualization
Add-Type -Path "$PSScriptRoot/Showcase.cs" -ReferencedAssemblies System.Drawing,System.Windows.Forms,System.Windows.Forms.DataVisualization
$rootPath=[IO.Path]::GetFullPath($Root)
$summary=Get-Content (Join-Path $rootPath 'docs/releases/0.2.0/performance.json') -Raw|ConvertFrom-Json
[double[]]$before=@(0,25,100,250)|ForEach-Object{$summary.builds.baseline.densities."$_".paintP95}
[double[]]$after=@(0,25,100,250)|ForEach-Object{$summary.builds.natural.densities."$_".paintP95}
[ReleaseShowcase]::Run($rootPath,$before,$after)
Get-ChildItem (Join-Path $rootPath 'docs/releases/0.2.0/*.png')|Select-Object Name,Length
