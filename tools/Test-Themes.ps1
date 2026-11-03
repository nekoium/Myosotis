param([string]$Exe = "$PSScriptRoot/../target/release/myosotis.exe")
# Interactive check: takes focus/pointer and replaces the clipboard with a synthetic white canvas.
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Windows.Forms, System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class ThemeCheck {
    [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern IntPtr FindWindow(string cls, IntPtr title);
    [DllImport("user32.dll")] public static extern IntPtr SendMessage(IntPtr hwnd, uint msg, IntPtr wp, IntPtr lp);
    [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr hwnd, uint msg, IntPtr wp, IntPtr lp);
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr hwnd);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hwnd);
    [DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr context);
    [StructLayout(LayoutKind.Sequential)] public struct Rect { public int Left, Top, Right, Bottom; }
    [DllImport("user32.dll")] public static extern bool GetMenuItemRect(IntPtr hwnd, IntPtr menu, uint item, out Rect rect);
    [DllImport("user32.dll")] public static extern void mouse_event(uint flags, uint x, uint y, uint data, UIntPtr extra);
    [DllImport("user32.dll")] public static extern void keybd_event(byte key, byte scan, uint flags, UIntPtr extra);
    [DllImport("user32.dll")] public static extern IntPtr GetSubMenu(IntPtr menu, int position);
    [DllImport("user32.dll")] public static extern uint GetMenuState(IntPtr menu, uint item, uint flags);
    [DllImport("user32.dll")] public static extern IntPtr GetDC(IntPtr hwnd);
    [DllImport("user32.dll")] public static extern int ReleaseDC(IntPtr hwnd, IntPtr dc);
    [DllImport("gdi32.dll")] public static extern uint GetPixel(IntPtr dc, int x, int y);
}
'@
[void][ThemeCheck]::SetProcessDpiAwarenessContext([IntPtr](-4))
function Assert($Condition, $Message) { if (!$Condition) { throw $Message } }
function Message([uint32]$Id, [int]$Value=0) {
    [void][ThemeCheck]::SendMessage($script:window,$Id,[IntPtr]$Value,[IntPtr]::Zero)
}
function Wait-For($Check, $Description) {
    for ($i=0; $i -lt 100; $i++) {
        if (& $Check) { return }
        Start-Sleep -Milliseconds 30
    }
    throw "Timed out: $Description"
}
function Start-App {
    $info = New-Object Diagnostics.ProcessStartInfo
    $info.FileName = [IO.Path]::GetFullPath($Exe)
    $info.UseShellExecute = $false
    $info.EnvironmentVariables['LOCALAPPDATA'] = $script:settingsRoot
    $script:process = [Diagnostics.Process]::Start($info)
    Wait-For {
        $script:window = [ThemeCheck]::FindWindow('Myosotis.Native.Overlay',[IntPtr]::Zero)
        $script:window -ne [IntPtr]::Zero
    } 'resident window'
    Assert ($script:process.WaitForInputIdle(3000)) 'App did not become ready for input'
}
function Stop-App {
    Message 0x100 27
    [void][ThemeCheck]::PostMessage($script:window,0x10,[IntPtr]::Zero,[IntPtr]::Zero)
    Assert ($script:process.WaitForExit(3000)) 'App failed to quit'
    $script:process.Dispose()
    $script:process = $null
}
function Open-Themes([int]$Selected) {
    Write-Output "Checking tray selection $Selected"
    [void][ThemeCheck]::SetCursorPos(80,80)
    [void][ThemeCheck]::PostMessage($script:window,0x8001,[IntPtr]::Zero,[IntPtr]0x205)
    Wait-For { [ThemeCheck]::FindWindow('#32768',[IntPtr]::Zero) -ne [IntPtr]::Zero } 'tray menu'
    $popup = [ThemeCheck]::FindWindow('#32768',[IntPtr]::Zero)
    $menu = [ThemeCheck]::SendMessage($popup,0x1E1,[IntPtr]::Zero,[IntPtr]::Zero)
    $themes = [ThemeCheck]::GetSubMenu($menu,2)
    Assert ($themes -ne [IntPtr]::Zero) 'Missing Theme submenu'
    $script:themeMenu = $themes
    $area = New-Object ThemeCheck+Rect
    Assert ([ThemeCheck]::GetMenuItemRect([IntPtr]::Zero,$menu,2,[ref]$area)) 'Cannot locate Theme submenu'
    $script:themeMenuX = [int](($area.Left+$area.Right)/2)
    $script:themeMenuY = [int](($area.Top+$area.Bottom)/2)
    foreach ($id in @(3,4)) {
        $state = [ThemeCheck]::GetMenuState($themes,$id,0)
        Assert ($state -ne [uint32]::MaxValue) 'Missing theme item'
        Assert ((($state -band 8) -ne 0) -eq ($id -eq $Selected)) 'Incorrect theme selection mark'
    }
}
function Menu-Key([byte]$Key) {
    [ThemeCheck]::keybd_event($Key,0,0,[UIntPtr]::Zero)
    [ThemeCheck]::keybd_event($Key,0,2,[UIntPtr]::Zero)
    Start-Sleep -Milliseconds 150
}
function Menu-Click {
    [ThemeCheck]::mouse_event(2,0,0,0,[UIntPtr]::Zero)
    Start-Sleep -Milliseconds 60
    [ThemeCheck]::mouse_event(4,0,0,0,[UIntPtr]::Zero)
    Start-Sleep -Milliseconds 150
}
function Choose-Theme([bool]$Light) {
    [void][ThemeCheck]::SetCursorPos($script:themeMenuX,$script:themeMenuY)
    Start-Sleep -Milliseconds 150
    Menu-Click
    $script:choiceArea = New-Object ThemeCheck+Rect
    $item = if ($Light) { 1 } else { 0 }
    Wait-For { [ThemeCheck]::GetMenuItemRect([IntPtr]::Zero,$script:themeMenu,$item,[ref]$script:choiceArea) } 'theme choices'
    [void][ThemeCheck]::SetCursorPos([int](($script:choiceArea.Left+$script:choiceArea.Right)/2),[int](($script:choiceArea.Top+$script:choiceArea.Bottom)/2))
    Start-Sleep -Milliseconds 150
    Menu-Click
    Wait-For { [ThemeCheck]::FindWindow('#32768',[IntPtr]::Zero) -eq [IntPtr]::Zero } 'menu dismissal'
}
function Capture-Canvas {
    [void][ThemeCheck]::PostMessage($script:window,0x312,[IntPtr]0x4D59,[IntPtr]::Zero)
    Wait-For { [ThemeCheck]::IsWindowVisible($script:window) } 'capture overlay'
    Start-Sleep -Milliseconds 200
    [void][ThemeCheck]::SetCursorPos(400,300); Message 0x200; Message 0x204
    [void][ThemeCheck]::SetCursorPos(1200,900); Message 0x200; Message 0x205
    [void][ThemeCheck]::SetCursorPos(600,500); Message 0x200; Message 0x201
    [void][ThemeCheck]::SetCursorPos(800,500); Message 0x200; Message 0x202
    [void][ThemeCheck]::SetCursorPos(1300,200); Message 0x200
}
function Assert-Panel([bool]$Light) {
    Start-Sleep -Milliseconds 150
    $scale = [ThemeCheck]::GetDpiForWindow($script:window) / 96.0
    $barX = [Math]::Max(12*$scale, [Math]::Min(400, $script:screen.Width-(240+12)*$scale))
    $barY = if (900+(44+57)*$scale -le $script:screen.Height) { 900+12*$scale } elseif (300-(44+12)*$scale -ge 0) { 300-(44+12)*$scale } else { 12*$scale }
    $dc = [ThemeCheck]::GetDC([IntPtr]::Zero)
    try { $pixel = [ThemeCheck]::GetPixel($dc,[int]($barX+2*$scale),[int]($barY+22*$scale)) }
    finally { [void][ThemeCheck]::ReleaseDC([IntPtr]::Zero,$dc) }
    $red = $pixel -band 255
    Assert ($(if ($Light) { $red -gt 220 } else { $red -lt 60 })) "Wrong toolbar palette: $pixel"
}
$existing = [ThemeCheck]::FindWindow('Myosotis.Native.Overlay',[IntPtr]::Zero)
Assert ($existing -eq [IntPtr]::Zero) 'Quit resident Myosotis before running this check.'
$script:screen = [Windows.Forms.Screen]::PrimaryScreen.Bounds
Assert ($screen.Width -ge 1400 -and $screen.Height -ge 1000 -and $screen.X -eq 0 -and $screen.Y -eq 0) 'Requires primary display at (0,0) of at least 1400 x 1000'
$script:settingsRoot = [IO.Path]::GetFullPath("$PSScriptRoot/../.tmp/theme-integration-$PID")
$settings = Join-Path $settingsRoot 'Myosotis/settings.toml'
$script:process = $null
$pointer = [Windows.Forms.Cursor]::Position
try {
    Assert (!(Test-Path $settingsRoot)) 'Test folder already exists'
    Start-App
    Assert (!(Test-Path $settingsRoot)) 'Default launch created settings'
    Open-Themes 3
    Choose-Theme $true
    Wait-For { Test-Path $settings } 'saved Light preference'
    Assert ((Get-Content $settings -Raw) -match 'theme\s*=\s*"light"') 'Light was not persisted'
    Stop-App
    Start-App
    Open-Themes 4
    Menu-Key 27
    Capture-Canvas
    Assert-Panel $true
    # Switch while the current blank canvas is active; its selection and export must survive.
    Open-Themes 4
    Choose-Theme $false
    Assert ([ThemeCheck]::IsWindowVisible($script:window)) 'Switch closed capture'
    Assert-Panel $false
    Open-Themes 3
    Choose-Theme $true
    Assert-Panel $true
    Message 0x100 0x5A
    Message 0x100 0x59
    Message 0x100 13
    Wait-For { ![ThemeCheck]::IsWindowVisible($script:window) } 'copy completion'
    $image = [Windows.Forms.Clipboard]::GetImage()
    try {
        Assert ($image.Width -eq 800 -and $image.Height -eq 600) "Theme switch changed selection: $($image.Width) x $($image.Height)"
        Assert ($image.GetPixel(20,20).ToArgb() -eq [Drawing.Color]::White.ToArgb()) 'Theme changed exported white canvas'
        $ink = $image.GetPixel(300,200)
        Assert ($ink.R -gt 200 -and $ink.G -lt 120 -and $ink.B -lt 150) 'Theme switch changed ink or lost annotation history'
    } finally { if ($null -ne $image) { $image.Dispose() } }
    Stop-App
    Write-Output 'PASS default without settings, native menu marks, persistence, live palettes, capture/history preservation, white canvas and coral clipboard output'
} finally {
    if ($null -ne $script:process) {
        if (!$script:process.HasExited) { $script:process.Kill(); $script:process.WaitForExit() }
        $script:process.Dispose()
    }
    [Windows.Forms.Cursor]::Position = $pointer
    if (Test-Path $settingsRoot) { Remove-Item -Recurse -Force $settingsRoot }
}
