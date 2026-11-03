param(
    [Parameter(Mandatory=$true)][string]$Exe,
    [Parameter(Mandatory=$true)][string]$Out,
    [ValidateSet('Light','Dark')][string]$Theme = 'Light'
)
# Draw a synthetic fixture and capture only its bounds. Never export desktop contents or change the clipboard.
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Windows.Forms, System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class DesignPreview {
    [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern IntPtr FindWindow(string cls, IntPtr title);
    [DllImport("user32.dll")] public static extern IntPtr SendMessage(IntPtr hwnd, uint msg, IntPtr wp, IntPtr lp);
    [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr hwnd, uint msg, IntPtr wp, IntPtr lp);
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hwnd);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hwnd);
    [DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr context);
}
'@
[void][DesignPreview]::SetProcessDpiAwarenessContext([IntPtr](-4))
$existing = [DesignPreview]::FindWindow('Myosotis.Native.Overlay', [IntPtr]::Zero)
if ($existing -ne [IntPtr]::Zero) { throw 'Quit the resident Myosotis before previewing a candidate.' }
$screen = [Windows.Forms.Screen]::PrimaryScreen.Bounds
if ($screen.Width -lt 1200 -or $screen.Height -lt 980) { throw 'Preview requires a primary display of at least 1200 x 980 physical pixels.' }
$outPath = [IO.Path]::GetFullPath($Out)
[void][IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($outPath))
$form = New-Object Windows.Forms.Form
$form.StartPosition = 'Manual'
$form.FormBorderStyle = 'None'
$form.Location = New-Object Drawing.Point ($screen.X+40),($screen.Y+40)
$form.ClientSize = New-Object Drawing.Size 1120,900
$dark = $Theme -eq 'Dark'
$form.BackColor = [Drawing.ColorTranslator]::FromHtml($(if ($dark) { '#202124' } else { '#edf0f3' }))
$form.TopMost = $true
$form.add_Paint({
    param($sender,$eventArgs)
    $g = $eventArgs.Graphics
    $g.SmoothingMode = 'AntiAlias'
    $title = New-Object Drawing.Font 'Segoe UI',22,([Drawing.FontStyle]::Bold),([Drawing.GraphicsUnit]::Pixel)
    $body = New-Object Drawing.Font 'Segoe UI',16,([Drawing.FontStyle]::Regular),([Drawing.GraphicsUnit]::Pixel)
    $small = New-Object Drawing.Font 'Segoe UI',14,([Drawing.FontStyle]::Regular),([Drawing.GraphicsUnit]::Pixel)
    $ink = New-Object Drawing.SolidBrush ([Drawing.ColorTranslator]::FromHtml($(if ($dark) { '#f1f3f4' } else { '#252a30' })))
    $muted = New-Object Drawing.SolidBrush ([Drawing.ColorTranslator]::FromHtml($(if ($dark) { '#bdc1c6' } else { '#59636d' })))
    $surface = New-Object Drawing.SolidBrush ([Drawing.ColorTranslator]::FromHtml($(if ($dark) { '#292a2d' } else { '#ffffff' })))
    $notes = New-Object Drawing.SolidBrush ([Drawing.ColorTranslator]::FromHtml($(if ($dark) { '#35363a' } else { '#f5f5f5' })))
    try {
        $g.DrawString('Myosotis', $title, $ink, 32, 20)
        $g.DrawString('Capture. Annotate. Keep moving.', $body, $muted, 215, 31)
        $g.FillRectangle($surface, 64, 100, 950, 440)
        $g.DrawString('Interface review', $title, $ink, 96, 120)
        $g.DrawString('A synthetic capture for comparing both native design directions.', $body, $muted, 96, 164)
        $g.DrawString('Clear controls', $body, $ink, 118, 232)
        $g.DrawString('Recognizable tools, one action to copy.', $small, $muted, 118, 264)
        $g.DrawString('Instant feedback', $body, $ink, 118, 342)
        $g.DrawString('Keep the capture and drawing path immediate.', $small, $muted, 118, 374)
        $g.FillRectangle($notes, 600, 224, 360, 245)
        $g.DrawString('Review notes', $body, $ink, 628, 247)
        $g.DrawString('Reduce persistent text', $small, $muted, 628, 296)
        $g.DrawString('Refine annotation defaults', $small, $muted, 628, 334)
        $g.DrawString('Preserve keyboard shortcuts', $small, $muted, 628, 372)
        $g.DrawString('Same scene and annotation geometry in each candidate.', $small, $muted, 64, 850)
    } finally { $title.Dispose(); $body.Dispose(); $small.Dispose(); $ink.Dispose(); $muted.Dispose(); $surface.Dispose(); $notes.Dispose() }
})
$process = $null
$oldPointer = [Windows.Forms.Cursor]::Position
try {
    $form.Show()
    [Windows.Forms.Application]::DoEvents()
    [void][DesignPreview]::SetForegroundWindow($form.Handle)
    Start-Sleep -Milliseconds 200
    $process = Start-Process ([IO.Path]::GetFullPath($Exe)) -PassThru -RedirectStandardError "$outPath.log"
    for ($i=0; $i -lt 100; $i++) {
        $script:previewWindow = [DesignPreview]::FindWindow('Myosotis.Native.Overlay', [IntPtr]::Zero)
        if ($script:previewWindow -ne [IntPtr]::Zero) { break }
        Start-Sleep -Milliseconds 30
    }
    if ($script:previewWindow -eq [IntPtr]::Zero) { throw 'Candidate did not start.' }
    [void][DesignPreview]::PostMessage($script:previewWindow,0x312,[IntPtr]0x4D59,[IntPtr]::Zero)
    for ($i=0; $i -lt 100; $i++) {
        if ([DesignPreview]::IsWindowVisible($script:previewWindow)) { break }
        Start-Sleep -Milliseconds 30
    }
    Start-Sleep -Milliseconds 250
    function Msg([uint32]$Id,[int]$Value=0) { [void][DesignPreview]::SendMessage($script:previewWindow,$Id,[IntPtr]$Value,[IntPtr]::Zero) }
    function Move-PreviewPointer([int]$X,[int]$Y) {
        [void][DesignPreview]::SetCursorPos(($screen.X+40+$X),($screen.Y+40+$Y))
        Msg 0x200
    }
    function Drag([int]$X1,[int]$Y1,[int]$X2,[int]$Y2) { Move-PreviewPointer $X1 $Y1; Msg 0x201; Move-PreviewPointer $X2 $Y2; Msg 0x202 }
    Drag 64 100 1014 540
    Move-PreviewPointer 116 302; Msg 0x201
    foreach ($p in @(@(155,307),@(196,300),@(239,304),@(282,298),@(329,302),@(365,299))) { Move-PreviewPointer $p[0] $p[1] }
    Msg 0x202
    Msg 0x100 0x32
    Drag 490 395 573 325
    Msg 0x100 0x33
    Drag 615 279 930 320
    Msg 0x100 0x34
    Move-PreviewPointer 575 248; Msg 0x201; Msg 0x202
    Msg 0x100 0x31
    Move-PreviewPointer 1060 670
    Start-Sleep -Milliseconds 150
    $bitmap = New-Object Drawing.Bitmap 1120,900
    $graphics = [Drawing.Graphics]::FromImage($bitmap)
    try {
        $graphics.CopyFromScreen($form.Location, [Drawing.Point]::Empty, $form.ClientSize)
        $bitmap.Save($outPath, [Drawing.Imaging.ImageFormat]::Png)
    } finally { $graphics.Dispose(); $bitmap.Dispose() }
    Msg 0x100 0x1B
    [void][DesignPreview]::PostMessage($script:previewWindow,0x10,[IntPtr]::Zero,[IntPtr]::Zero)
    if (!$process.WaitForExit(3000)) { throw 'Candidate did not close.' }
    Write-Output "Saved $outPath"
} finally {
    if ($null -ne $process -and !$process.HasExited) { $process.Kill(); $process.WaitForExit() }
    $form.Dispose()
    [Windows.Forms.Cursor]::Position = $oldPointer
}
