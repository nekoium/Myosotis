param([string]$OutputPath = "$PSScriptRoot/../.tmp/clipboard.png")
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class ClipboardProbe {
    [DllImport("user32.dll")] public static extern bool OpenClipboard(IntPtr hwnd);
    [DllImport("user32.dll")] public static extern bool CloseClipboard();
    [DllImport("user32.dll")] public static extern IntPtr GetClipboardData(uint format);
    [DllImport("kernel32.dll")] public static extern IntPtr GlobalLock(IntPtr handle);
    [DllImport("kernel32.dll")] public static extern bool GlobalUnlock(IntPtr handle);
    [DllImport("kernel32.dll")] public static extern UIntPtr GlobalSize(IntPtr handle);
}
'@
if (![ClipboardProbe]::OpenClipboard([IntPtr]::Zero)) { throw 'Clipboard busy' }
try {
    $handle = [ClipboardProbe]::GetClipboardData(8)
    if ($handle -eq [IntPtr]::Zero) { throw 'Clipboard does not contain CF_DIB' }
    $pointer = [ClipboardProbe]::GlobalLock($handle)
    if ($pointer -eq [IntPtr]::Zero) { throw 'Could not lock clipboard DIB' }
    try {
        $length = [ClipboardProbe]::GlobalSize($handle).ToUInt64()
        $bytes = New-Object byte[] $length
        [Runtime.InteropServices.Marshal]::Copy($pointer, $bytes, 0, $bytes.Length)
    } finally { [void][ClipboardProbe]::GlobalUnlock($handle) }
} finally { [void][ClipboardProbe]::CloseClipboard() }
$width = [BitConverter]::ToInt32($bytes, 4)
$height = [BitConverter]::ToInt32($bytes, 8)
Write-Output "DIB: $width x $height; $($bytes.Length) bytes; header $([BitConverter]::ToUInt32($bytes,0))"
$stream = New-Object IO.MemoryStream
$writer = New-Object IO.BinaryWriter $stream
$writer.Write([byte[]](66,77))
$writer.Write([uint32]($bytes.Length+14))
$writer.Write([uint32]0)
$writer.Write([uint32]54)
$writer.Write($bytes)
$writer.Flush()
$stream.Position = 0
$image = [Drawing.Bitmap]::FromStream($stream)
try {
    $fullOutputPath = [IO.Path]::GetFullPath($OutputPath)
    [void][IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($fullOutputPath))
    $image.Save($fullOutputPath, [Drawing.Imaging.ImageFormat]::Png)
    Write-Output "Decoded bitmap: $($image.Width) x $($image.Height), saved to $OutputPath"
} finally { $image.Dispose(); $writer.Dispose(); $stream.Dispose() }
