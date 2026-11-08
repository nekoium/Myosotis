# Regenerate the selected flat flower (direction A) using system drawing facilities.
# Palette and geometry: docs/ICON-DESIGN.md.
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
$colors = @('#648DF0', '#7EA3F8', '#6B95F2', '#86AAF7', '#739CF4')
$bitmap = New-Object Drawing.Bitmap 1024,1024
$g = [Drawing.Graphics]::FromImage($bitmap)
$g.SmoothingMode = [Drawing.Drawing2D.SmoothingMode]::AntiAlias
try {
    for ($i=0; $i -lt 5; $i++) {
        $g.ResetTransform()
        $g.ScaleTransform(4,4)
        $g.TranslateTransform(128,128)
        $g.ScaleTransform(1.16,1.16)
        $g.RotateTransform($i*72)
        $petal = New-Object Drawing.SolidBrush ([Drawing.ColorTranslator]::FromHtml($colors[$i]))
        $path = New-Object Drawing.Drawing2D.GraphicsPath
        try {
            $path.AddBezier(-13,-4,-48,-28,-51,-62,-34,-87)
            $path.AddBezier(-34,-87,-13,-115,35,-103,41,-76)
            $path.AddBezier(41,-76,49,-48,28,-12,-13,-4)
            $path.CloseFigure()
            $g.FillPath($petal,$path)
        } finally { $path.Dispose();$petal.Dispose() }
    }
    $g.ResetTransform()
    $g.ScaleTransform(4,4)
    $g.TranslateTransform(128,128)
    $g.ScaleTransform(1.16,1.16)
    $center = New-Object Drawing.SolidBrush ([Drawing.ColorTranslator]::FromHtml('#F5C85C'))
    try { $g.FillEllipse($center,-23,-23,46,46) } finally { $center.Dispose() }
} finally { $g.Dispose() }
$images = @()
try {
    foreach ($size in @(16,20,24,32,40,48,64,96,128,256)) {
        $scaled = New-Object Drawing.Bitmap $size,$size
        $sg = [Drawing.Graphics]::FromImage($scaled)
        $stream = New-Object IO.MemoryStream
        try {
            $sg.InterpolationMode = [Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
            $sg.DrawImage($bitmap,0,0,$size,$size)
            $scaled.Save($stream,[Drawing.Imaging.ImageFormat]::Png)
            $images += @{Size=$size;Data=$stream.ToArray()}
            if ($size -eq 256) {
                $scaled.Save("$PSScriptRoot/../assets/myosotis.png",[Drawing.Imaging.ImageFormat]::Png)
            }
        } finally { $stream.Dispose();$sg.Dispose();$scaled.Dispose() }
    }
} finally { $bitmap.Dispose() }
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

# Export the same paths and palette for sharp README rendering at any scale.
$svg = @('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256">',
    '  <g transform="translate(128 128) scale(1.16)">')
for ($i=0; $i -lt 5; $i++) {
    $angle = $i*72
    $svg += "    <path fill=""$($colors[$i])"" transform=""rotate($angle)"" d=""M-13-4C-48-28-51-62-34-87C-13-115 35-103 41-76C49-48 28-12-13-4Z""/>"
}
$svg += @('    <circle r="23" fill="#F5C85C"/>', '  </g>', '</svg>')
[IO.File]::WriteAllText("$PSScriptRoot/../assets/myosotis.svg", ($svg -join "`n") + "`n")
