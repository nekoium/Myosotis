# Regenerate the multi-resolution Windows icon with system drawing facilities.
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
$images = @()
foreach ($size in @(16,32,48,256)) {
    $bitmap = New-Object Drawing.Bitmap 256,256
    $g = [Drawing.Graphics]::FromImage($bitmap)
    $g.SmoothingMode = [Drawing.Drawing2D.SmoothingMode]::AntiAlias
    $petal = New-Object Drawing.SolidBrush ([Drawing.Color]::FromArgb(139,170,255))
    $center = New-Object Drawing.SolidBrush ([Drawing.Color]::FromArgb(255,213,94))
    $outline = New-Object Drawing.Pen ([Drawing.Color]::FromArgb(30,44,86)),8
    for ($i=0; $i -lt 5; $i++) {
        $angle = (-90+$i*72)*[Math]::PI/180
        $x = [single](128+62*[Math]::Cos($angle)-48)
        $y = [single](128+62*[Math]::Sin($angle)-48)
        $g.FillEllipse($petal,$x,$y,96,96)
        $g.DrawEllipse($outline,$x,$y,96,96)
    }
    $g.FillEllipse($center,100,100,56,56)
    $g.DrawEllipse($outline,100,100,56,56)
    $scaled = New-Object Drawing.Bitmap $size,$size
    $sg = [Drawing.Graphics]::FromImage($scaled)
    $sg.InterpolationMode = [Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $sg.DrawImage($bitmap,0,0,$size,$size)
    $stream = New-Object IO.MemoryStream
    $scaled.Save($stream,[Drawing.Imaging.ImageFormat]::Png)
    $images += @{Size=$size;Data=$stream.ToArray()}
    $stream.Dispose();$sg.Dispose();$scaled.Dispose();$g.Dispose();$bitmap.Dispose();$petal.Dispose();$center.Dispose();$outline.Dispose()
}
$file = [IO.File]::Create("$PSScriptRoot/../assets/myosotis.ico")
$writer = New-Object IO.BinaryWriter $file
try {
    $writer.Write([uint16]0);$writer.Write([uint16]1);$writer.Write([uint16]$images.Count)
    $offset = 6+16*$images.Count
    foreach ($image in $images) {
        $sizeByte = $image.Size % 256
        $writer.Write([byte]$sizeByte);$writer.Write([byte]$sizeByte);$writer.Write([uint16]0)
        $writer.Write([uint16]1);$writer.Write([uint16]32)
        $writer.Write([uint32]$image.Data.Length);$writer.Write([uint32]$offset)
        $offset += $image.Data.Length
    }
    foreach ($image in $images) { $writer.Write([byte[]]$image.Data) }
} finally { $writer.Dispose();$file.Dispose() }
