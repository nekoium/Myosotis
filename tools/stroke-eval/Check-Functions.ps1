param([Parameter(Mandatory=$true)][string]$Exe, [switch]$Shapes, [switch]$History, [string]$ResultName = 'stroke-functions', [int]$Repeat = 1, [switch]$Interactive)
$ErrorActionPreference = 'Stop'
if (!$Interactive) { throw 'These checks own the desktop cursor, focus, and clipboard for their entire run. Use cargo test --locked for cursor-free checks. Pass -Interactive only when the desktop is available.' }
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
# Materialize clipboard formats before the smoke check replaces their owner.
$original = [Windows.Forms.Clipboard]::GetDataObject()
$backup = New-Object Windows.Forms.DataObject
if ($null -ne $original) {
    foreach ($format in $original.GetFormats($false)) {
        $value = $original.GetData($format,$false)
        if ($value -is [IO.MemoryStream]) { $value = New-Object IO.MemoryStream(,$value.ToArray()) }
        elseif ($value -is [ICloneable]) { $value = $value.Clone() }
        if ($null -ne $value) { $backup.SetData($format,$false,$value) }
    }
}
try {
    if ($Shapes -or $History) {
        $density = -2
        if ($History) { $density = -4 }
        & "$PSScriptRoot/Run.ps1" -Executable $Exe -ResultName $ResultName -Densities @($density) -Repeat $Repeat -Interactive
    } else {
        & "$PSScriptRoot/../Smoke-Test.ps1" -Exe $Exe -Interactive
    }
} finally {
    [Windows.Forms.Clipboard]::SetDataObject($backup,$true)
}
