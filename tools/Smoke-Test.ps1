param(
    [string]$Exe = "$PSScriptRoot/../dist/Myosotis/myosotis.exe",
    [float]$ToolbarCell = 40,
    [float]$ToolbarHeight = 44,
    [switch]$Interactive
)
# Interactive Windows check: takes pointer/focus and replaces the clipboard with test content.
$ErrorActionPreference = 'Stop'
if (!$Interactive) { throw 'This suite owns the desktop cursor, focus, and clipboard for its entire run. Use cargo test --locked for cursor-free checks. Pass -Interactive only when the desktop is available.' }
Add-Type -AssemblyName System.Windows.Forms
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class NativeSmoke {
    [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern IntPtr FindWindow(string cls, IntPtr title);
    [DllImport("user32.dll")] public static extern IntPtr SendMessage(IntPtr hwnd, uint msg, IntPtr wp, IntPtr lp);
    [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr hwnd, uint msg, IntPtr wp, IntPtr lp);
    [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")] public static extern bool OpenClipboard(IntPtr hwnd);
    [DllImport("user32.dll")] public static extern bool CloseClipboard();
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hwnd);
    [DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr hwnd);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hwnd);
    [DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr context);
    [DllImport("user32.dll")] public static extern void keybd_event(byte vk, byte scan, uint flags, UIntPtr extra);
}
'@
[void][NativeSmoke]::SetProcessDpiAwarenessContext([IntPtr](-4))
function Assert($Condition, $Message) { if (!$Condition) { throw $Message } }
function Wait-Visible($Expected) {
    for ($i=0; $i -lt 100; $i++) {
        if ([NativeSmoke]::IsWindowVisible($script:window) -eq $Expected) { return }
        Start-Sleep -Milliseconds 30
    }
    throw "Overlay visibility did not become $Expected"
}
function Message([uint32]$Id, [int]$Value=0) { [void][NativeSmoke]::SendMessage($script:window,$Id,[IntPtr]$Value,[IntPtr]::Zero) }
function Key([int]$Value) { Message 0x100 $Value; Message 0x101 $Value }
function Move-Pointer([int]$X,[int]$Y) { [void][NativeSmoke]::SetCursorPos($X,$Y); Message 0x200 }
function Drag([int]$X1,[int]$Y1,[int]$X2,[int]$Y2,[bool]$Blank=$false) {
    Move-Pointer $X1 $Y1
    if ($Blank) { Message 0x204 } else { Message 0x201 }
    Move-Pointer $X2 $Y2
    if ($Blank) { Message 0x205 } else { Message 0x202 }
}
function Tap([int]$X,[int]$Y,[bool]$Space=$false) {
    Move-Pointer $X $Y
    if ($Space) { Key 0x20 } else { Message 0x201; Message 0x202 }
}
function Capture {
    # Exercise the registered system shortcut, rather than calling the application's activation message.
    [NativeSmoke]::keybd_event(0x11,0,0,[UIntPtr]::Zero)
    [NativeSmoke]::keybd_event(0x12,0,0,[UIntPtr]::Zero)
    [NativeSmoke]::keybd_event(0x53,0,0,[UIntPtr]::Zero)
    [NativeSmoke]::keybd_event(0x53,0,2,[UIntPtr]::Zero)
    [NativeSmoke]::keybd_event(0x12,0,2,[UIntPtr]::Zero)
    [NativeSmoke]::keybd_event(0x11,0,2,[UIntPtr]::Zero)
    Wait-Visible $true
    [void][NativeSmoke]::SetForegroundWindow($script:window)
    # Visibility changes before activation finishes; do not inject synchronous input during setup.
    Start-Sleep -Milliseconds 150
}
function Copy-Image {
    Key 13
    Wait-Visible $false
    for ($i=0; $i -lt 20; $i++) {
        $image = [System.Windows.Forms.Clipboard]::GetImage()
        if ($null -ne $image) { return $image }
        Start-Sleep -Milliseconds 50
    }
    throw 'Clipboard image could not be read by Windows Forms'
}
function Is-Ink($Image,[int]$X,[int]$Y) {
    $p = $Image.GetPixel($X,$Y)
    return [Math]::Abs($p.R-199) -le 12 -and [Math]::Abs($p.G-70) -le 12 -and [Math]::Abs($p.B-86) -le 12
}
$existing = [NativeSmoke]::FindWindow('Myosotis.Native.Overlay',[IntPtr]::Zero)
Assert ($existing -eq [IntPtr]::Zero) 'Quit the resident Myosotis before running this check.'
$logFolder = [IO.Path]::GetFullPath("$PSScriptRoot/../.tmp")
[void][IO.Directory]::CreateDirectory($logFolder)
$process = Start-Process ([IO.Path]::GetFullPath($Exe)) -PassThru -RedirectStandardError "$logFolder/smoke-timings.log"
try {
    for ($i=0; $i -lt 100; $i++) {
        $script:window = [NativeSmoke]::FindWindow('Myosotis.Native.Overlay',[IntPtr]::Zero)
        if ($script:window -ne [IntPtr]::Zero) { break }
        Start-Sleep -Milliseconds 30
    }
    Assert ($script:window -ne [IntPtr]::Zero) 'App did not create its resident window'
    $screen = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
    Assert ($screen.Width -ge 1400 -and $screen.Height -ge 1000 -and $screen.X -eq 0 -and $screen.Y -eq 0) 'Smoke check needs a primary display at (0,0) of at least 1400 x 1000 physical pixels'
    [System.Windows.Forms.Clipboard]::SetText('Myosotis cancellation sentinel')
    Capture
    Key 27
    Wait-Visible $false
    Assert ([System.Windows.Forms.Clipboard]::GetText() -eq 'Myosotis cancellation sentinel') 'Cancel replaced clipboard'
    Write-Output 'PASS global shortcut, Escape, clipboard preservation'

    # Both first-stage input sources can select and immediately finish a monitor capture.
    foreach ($spaceInput in @($false, $true)) {
        Capture
        Tap 700 450 $spaceInput
        Assert ([NativeSmoke]::IsWindowVisible($script:window)) 'First tap should select, not finish'
        Start-Sleep -Milliseconds 80
        Tap 700 450 $spaceInput
        Wait-Visible $false
        $image = [System.Windows.Forms.Clipboard]::GetImage()
        try { Assert ($image.Width -eq $screen.Width -and $image.Height -eq $screen.Height) 'Direct monitor copy dimensions incorrect' }
        finally { $image.Dispose() }
    }
    Write-Output 'PASS mouse and Space double-tap direct monitor capture'

    # Outside completion, separated sources, slow taps, toolbar exclusion, and Ctrl.
    Capture
    Drag 400 300 1200 900 $true
    Tap 250 200
    Start-Sleep -Milliseconds 80
    Tap 250 200
    Wait-Visible $false
    $image = [System.Windows.Forms.Clipboard]::GetImage()
    try {
        Assert ($image.Width -eq 800 -and $image.Height -eq 600) "Outside double-click changed selection: $($image.Width) x $($image.Height)"
        Assert ($image.GetPixel(20,20).ToArgb() -eq [Drawing.Color]::White.ToArgb()) 'Outside double-click changed background'
    } finally { $image.Dispose() }
    Capture
    Drag 400 300 1200 900 $true
    Tap 600 500
    Tap 600 500 $true
    Assert ([NativeSmoke]::IsWindowVisible($script:window)) 'Mixed input sources combined into completion'
    Start-Sleep -Milliseconds 300
    Tap 600 500 $true
    Assert ([NativeSmoke]::IsWindowVisible($script:window)) 'Slow taps completed the capture'
    Key 0x11
    Wait-Visible $false
    Write-Output 'PASS outside completion, separate sources, 250 ms cutoff, and immediate Ctrl'

    # A numbered first click is visible immediately, but never enters a completed export.
    foreach ($spaceInput in @($false, $true)) {
        Capture
        Drag 400 300 1200 900 $true
        Key 0x34
        Tap 600 500 $spaceInput
        Start-Sleep -Milliseconds 80
        Tap 600 500 $spaceInput
        Wait-Visible $false
        $image = [System.Windows.Forms.Clipboard]::GetImage()
        try {
            for ($x=180; $x -le 220; $x+=5) {
                for ($y=180; $y -le 220; $y+=5) {
                    Assert ($image.GetPixel($x,$y).ToArgb() -eq [Drawing.Color]::White.ToArgb()) 'Double-click exported a stray mark'
                }
            }
        } finally { $image.Dispose() }
    }
    Write-Output 'PASS mouse and Space numbered-mark rollback'

    Capture
    Drag 400 300 1200 900 $true
    Tap 400 600
    Start-Sleep -Milliseconds 80
    Tap 400 600
    Wait-Visible $false
    Write-Output 'PASS border double-click before resize hover delay'

    Capture
    Drag 400 300 1200 900 $true
    $scale = [NativeSmoke]::GetDpiForWindow($script:window) / 96.0
    $barX = [Math]::Max(12*$scale, [Math]::Min(400, $screen.Width-(6*$ToolbarCell+12)*$scale))
    $barY = if (900+($ToolbarHeight+57)*$scale -le $screen.Height) { 900+12*$scale } elseif (300-($ToolbarHeight+12)*$scale -ge 0) { 300-($ToolbarHeight+12)*$scale } else { 12*$scale }
    Tap ([int]($barX+$ToolbarCell/2*$scale)) ([int]($barY+$ToolbarHeight/2*$scale))
    Start-Sleep -Milliseconds 80
    Tap ([int]($barX+$ToolbarCell/2*$scale)) ([int]($barY+$ToolbarHeight/2*$scale))
    Assert ([NativeSmoke]::IsWindowVisible($script:window)) 'Toolbar double-click unexpectedly completed capture'
    Key 27
    Write-Output 'PASS toolbar double-click exclusion'

    Capture
    Drag 400 300 1200 900 $true
    Key 0x34
    Assert ([NativeSmoke]::OpenClipboard([IntPtr]::Zero)) 'Could not lock clipboard for double-click recovery check'
    try {
        Tap 600 500
        Start-Sleep -Milliseconds 80
        Tap 600 500
        Assert ([NativeSmoke]::IsWindowVisible($script:window)) 'Failed double-click copy discarded capture'
    } finally { [void][NativeSmoke]::CloseClipboard() }
    $image = Copy-Image
    try { Assert ($image.GetPixel(210,200).ToArgb() -eq [Drawing.Color]::White.ToArgb()) 'Failed copy recovery retained a completion mark' }
    finally { $image.Dispose() }
    Write-Output 'PASS double-click clipboard failure recovery without stray mark'

    Capture
    Drag 400 300 1200 900 $true
    Drag 500 500 700 500
    Key 0x52
    Key 0x5A
    Key 0x59
    $image = Copy-Image
    try {
        Assert ($image.Width -eq 800 -and $image.Height -eq 600) 'Reset changed selection'
        Assert ($image.GetPixel(200,200).ToArgb() -eq [Drawing.Color]::White.ToArgb()) 'Reset redo retained ink or changed background'
    } finally { $image.Dispose() }
    Capture
    Drag 400 300 1200 900 $true
    Drag 500 500 700 500
    Key 0x52
    Key 0x5A
    $image = Copy-Image
    try { Assert (Is-Ink $image 200 200) 'Reset undo failed to restore annotations' }
    finally { $image.Dispose() }
    Write-Output 'PASS reset selection/background preservation and undo/redo'

    Capture
    Drag 400 300 1200 900 $true
    Drag 450 400 650 400
    Key 0x32
    Drag 500 500 800 600
    Key 0x33
    Drag 900 400 1100 600
    Key 0x34
    Move-Pointer 1000 750; Message 0x201; Message 0x202
    Move-Pointer 800 750; Message 0x100 0x20
    [void][NativeSmoke]::SendMessage($script:window,0x100,[IntPtr]0x20,[IntPtr]0x40000000)
    Message 0x101 0x20
    Key 0x5A; Key 0x59
    Move-Pointer 400 600; Start-Sleep -Milliseconds 410
    Message 0x201; Move-Pointer 350 600; Message 0x202
    Move-Pointer 1200 600; Start-Sleep -Milliseconds 410
    Message 0x201; Move-Pointer 1250 600; Message 0x202
    $image = Copy-Image
    try {
        Assert ($image.Width -eq 900 -and $image.Height -eq 600) "Resized export is $($image.Width) x $($image.Height), expected 900 x 600"
        Assert (Is-Ink $image 150 100) 'Pen missing after resize'
        Assert (Is-Ink $image 225 225) 'Arrow missing'
        Assert (Is-Ink $image 550 150) 'Rectangle missing'
        Assert (Is-Ink $image 660 450) 'Numbered mark missing'
        Assert (Is-Ink $image 460 450) 'Space mark or redo missing'
        Assert ($image.GetPixel(1,1).ToArgb() -eq [Drawing.Color]::White.ToArgb()) 'Blank canvas not white'
        $data = [System.Windows.Forms.Clipboard]::GetDataObject()
        Assert ($data.GetDataPresent('PNG')) 'PNG clipboard representation missing'
        $pngStream = $data.GetData('PNG')
        $pngImage = [Drawing.Bitmap]::FromStream($pngStream)
        try {
            Assert ($pngImage.Width -eq 900 -and $pngImage.Height -eq 600) 'PNG dimensions differ from bitmap'
            Assert ($pngImage.GetPixel(1,1).A -eq 255) 'PNG has unexpected transparency'
            Assert (Is-Ink $pngImage 150 100) 'PNG lost annotations'
        } finally { $pngImage.Dispose(); $pngStream.Dispose() }
    } finally { $image.Dispose() }
    Write-Output 'PASS blank region, four tools, numbered Space action, undo/redo, delayed resize, DIB and PNG clipboard pixels'

    Capture
    Drag 400 300 1200 900 $true
    Key 0x31
    Move-Pointer 500 500; Message 0x100 0x20; Move-Pointer 700 500; Message 0x101 0x20
    Key 0x5A
    $image = Copy-Image
    try { Assert ($image.GetPixel(200,200).ToArgb() -eq [Drawing.Color]::White.ToArgb()) 'Undo did not remove Space pen' }
    finally { $image.Dispose() }
    Capture
    Drag 400 300 1200 900 $true
    Move-Pointer 500 500; Message 0x100 0x20; Move-Pointer 700 500; Message 0x101 0x20
    $image = Copy-Image
    try { Assert (Is-Ink $image 200 200) 'Held Space pen missing' }
    finally { $image.Dispose() }
    Write-Output 'PASS held Space pen and undo'

    Capture
    Move-Pointer 700 450; Message 0x204; Message 0x205
    $image = Copy-Image
    try { Assert ($image.Width -eq $screen.Width -and $image.Height -eq $screen.Height) 'Monitor canvas dimensions incorrect' }
    finally { $image.Dispose() }
    Capture
    Drag 400 300 1200 900
    Assert ([NativeSmoke]::OpenClipboard([IntPtr]::Zero)) 'Could not lock clipboard for contention check'
    try {
        Key 13
        Assert ([NativeSmoke]::IsWindowVisible($script:window)) 'Busy clipboard discarded the overlay'
    } finally { [void][NativeSmoke]::CloseClipboard() }
    $image = Copy-Image
    try { Assert ($image.Width -eq 800 -and $image.Height -eq 600) "Screenshot region dimensions incorrect: $($image.Width) x $($image.Height)" }
    finally { $image.Dispose() }
    Capture
    Move-Pointer 700 450; Message 0x201; Message 0x202
    $image = Copy-Image
    try { Assert ($image.Width -eq $screen.Width -and $image.Height -eq $screen.Height) 'Monitor screenshot dimensions incorrect' }
    finally { $image.Dispose() }
    Write-Output 'PASS monitor capture, screenshot crop, clipboard contention and retry'

    $fixture = New-Object System.Windows.Forms.Form
    try {
        $fixture.StartPosition = 'Manual'
        $fixture.FormBorderStyle = 'None'
        $fixture.Location = New-Object Drawing.Point 450,350
        $fixture.ClientSize = New-Object Drawing.Size 200,200
        $fixture.BackColor = [Drawing.Color]::CornflowerBlue
        $fixture.Show()
        [System.Windows.Forms.Application]::DoEvents()
        Assert ([NativeSmoke]::SetForegroundWindow($fixture.Handle)) 'Test fixture could not take foreground focus'
        Start-Sleep -Milliseconds 200
        Assert ([NativeSmoke]::GetForegroundWindow() -eq $fixture.Handle) 'Test fixture did not retain foreground focus'
        Capture
        $fixture.BackColor = [Drawing.Color]::OrangeRed
        $fixture.Refresh()
        Drag 400 300 1200 900
        $image = Copy-Image
        try { Assert ($image.GetPixel(100,100).ToArgb() -eq [Drawing.Color]::CornflowerBlue.ToArgb()) 'Capture did not freeze the original pixels' }
        finally { $image.Dispose() }
        Capture
        Drag 400 300 1200 900
        Drag 700 600 900 600
        Move-Pointer 750 550; Message 0x204; Message 0x205
        Key 0x52; Key 0x5A
        $image = Copy-Image
        try {
            Assert ($image.GetPixel(100,100).ToArgb() -eq [Drawing.Color]::White.ToArgb()) 'Right click did not switch screenshot to white'
            Assert (Is-Ink $image 400 300) 'Background toggle or reset undo lost annotations'
        } finally { $image.Dispose() }
        Capture
        Drag 400 300 1200 900 $true
        Drag 700 600 900 600
        Move-Pointer 750 550; Message 0x204; Message 0x205
        $image = Copy-Image
        try {
            Assert ($image.GetPixel(100,100).ToArgb() -eq [Drawing.Color]::OrangeRed.ToArgb()) 'Right click did not restore captured screenshot'
            Assert (Is-Ink $image 400 300) 'White to screenshot toggle lost annotations'
        } finally { $image.Dispose() }
        Write-Output 'PASS bidirectional background toggle preserving selection and annotations'
        for ($i=0; $i -lt 30 -and [NativeSmoke]::GetForegroundWindow() -ne $fixture.Handle; $i++) {
            [System.Windows.Forms.Application]::DoEvents()
            Start-Sleep -Milliseconds 10
        }
        Assert ([NativeSmoke]::GetForegroundWindow() -eq $fixture.Handle) "Previous window focus was not restored: expected $($fixture.Handle), got $([NativeSmoke]::GetForegroundWindow())"
    } finally { $fixture.Dispose() }
    Write-Output 'PASS frozen-screen pixels and previous-window focus restoration'
    for ($i=0; $i -lt 6; $i++) { Capture; Key 27; Wait-Visible $false }
    $process.Refresh()
    Write-Output "PASS repeated invocation; resident working set $([Math]::Round($process.WorkingSet64/1MB,1)) MiB; handles $($process.HandleCount)"
    [void][NativeSmoke]::PostMessage($script:window,0x10,[IntPtr]::Zero,[IntPtr]::Zero)
    Assert ($process.WaitForExit(3000)) 'Resident process did not quit cleanly'
    Write-Output 'PASS shutdown and tray cleanup'
} finally {
    if (!$process.HasExited) { $process.Kill(); $process.WaitForExit() }
}
