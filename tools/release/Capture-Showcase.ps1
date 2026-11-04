param(
    [Parameter(Mandatory=$true)][string]$Exe,
    [Parameter(Mandatory=$true)][string]$OutDir,
    [ValidateSet('Dark','Light')][string]$Theme = 'Dark'
)
# Native captures over a synthetic fixture only. Isolated settings; no clipboard writes.
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Windows.Forms,System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class ReleaseCapture {
 [DllImport("user32.dll",CharSet=CharSet.Unicode)] public static extern IntPtr FindWindow(string c,IntPtr t);
 [DllImport("user32.dll")] public static extern IntPtr SendMessage(IntPtr h,uint m,IntPtr w,IntPtr l);
 [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h,uint m,IntPtr w,IntPtr l);
 [DllImport("user32.dll")] public static extern bool SetCursorPos(int x,int y);
 [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
 [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
 [DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr c);
}
'@
[void][ReleaseCapture]::SetProcessDpiAwarenessContext([IntPtr](-4))
if ([ReleaseCapture]::FindWindow('Myosotis.Native.Overlay',[IntPtr]::Zero) -ne [IntPtr]::Zero) { throw 'Existing resident app must be closed before capture.' }
$screen = [Windows.Forms.Screen]::PrimaryScreen.Bounds
if ($screen.Width -lt 1520 -or $screen.Height -lt 1160) { throw 'Capture needs at least 1520 x 1160 physical pixels.' }
$outPath = [IO.Path]::GetFullPath($OutDir)
[void][IO.Directory]::CreateDirectory($outPath)
$settings = Join-Path $outPath 'settings-isolation'
[void][IO.Directory]::CreateDirectory((Join-Path $settings 'Myosotis'))
[IO.File]::WriteAllText((Join-Path $settings 'Myosotis/settings.toml'), "theme = `"$($Theme.ToLowerInvariant())`"`n")
$form = New-Object Windows.Forms.Form
$form.StartPosition='Manual'; $form.FormBorderStyle='None'
$form.Location=New-Object Drawing.Point 40,40
$form.ClientSize=New-Object Drawing.Size 1480,1100
$form.TopMost=$true; $form.BackColor=[Drawing.ColorTranslator]::FromHtml('#E9ECF1')
$form.add_Paint({
 param($sender,$e)
 $g=$e.Graphics; $g.SmoothingMode='AntiAlias'
 $title=New-Object Drawing.Font 'Segoe UI',30,([Drawing.FontStyle]::Bold),([Drawing.GraphicsUnit]::Pixel)
 $body=New-Object Drawing.Font 'Segoe UI',22,([Drawing.FontStyle]::Regular),([Drawing.GraphicsUnit]::Pixel)
 $small=New-Object Drawing.Font 'Segoe UI',18,([Drawing.FontStyle]::Regular),([Drawing.GraphicsUnit]::Pixel)
 $ink=New-Object Drawing.SolidBrush ([Drawing.ColorTranslator]::FromHtml('#252936'))
 $muted=New-Object Drawing.SolidBrush ([Drawing.ColorTranslator]::FromHtml('#687387'))
 $surface=New-Object Drawing.SolidBrush ([Drawing.ColorTranslator]::FromHtml('#F3F5F8'))
 try {
  $g.FillRectangle([Drawing.Brushes]::White,64,100,1350,670)
  $g.DrawString('A little feedback goes a long way.', $title,$ink,108,139)
  $g.DrawString('Capture a detail. Make your point. Keep moving.', $body,$muted,108,191)
  $g.FillRectangle($surface,108,250,660,444)
  $g.DrawString('Make this easier to find', $body,$ink,140,276)
  $g.DrawString('A quick underline, a loop, an arrow.', $small,$muted,140,390)
  $g.DrawString('Keep the original path and intentional corners.', $small,$muted,140,647)
  $g.DrawString('One point, or twenty.', $body,$ink,828,275)
  $g.DrawString('Numbered marks stay easy to read.', $small,$muted,828,315)
 } finally {$title.Dispose();$body.Dispose();$small.Dispose();$ink.Dispose();$muted.Dispose();$surface.Dispose()}
})
$process=$null; $oldPointer=[Windows.Forms.Cursor]::Position; $oldLocal=$env:LOCALAPPDATA
try {
 $env:LOCALAPPDATA=$settings
 $form.Show(); [Windows.Forms.Application]::DoEvents()
 [void][ReleaseCapture]::SetForegroundWindow($form.Handle)
 Start-Sleep -Milliseconds 150
 $process=Start-Process ([IO.Path]::GetFullPath($Exe)) -PassThru -RedirectStandardError (Join-Path $outPath 'capture.log')
 $script:captureHwnd=[IntPtr]::Zero
 for($i=0;$i -lt 100;$i++) { $script:captureHwnd=[ReleaseCapture]::FindWindow('Myosotis.Native.Overlay',[IntPtr]::Zero); if($script:captureHwnd -ne [IntPtr]::Zero){break}; Start-Sleep -Milliseconds 30 }
 if($script:captureHwnd -eq [IntPtr]::Zero){throw 'App did not start.'}
 [void][ReleaseCapture]::PostMessage($script:captureHwnd,0x312,[IntPtr]0x4D59,[IntPtr]::Zero)
 for($i=0;$i -lt 100;$i++){if([ReleaseCapture]::IsWindowVisible($script:captureHwnd)){break};Start-Sleep -Milliseconds 30}
 Start-Sleep -Milliseconds 150
 function Msg([uint32]$m,[int]$v=0){[void][ReleaseCapture]::SendMessage($script:captureHwnd,$m,[IntPtr]$v,[IntPtr]::Zero)}
 function Move-ShowcasePointer([int]$x,[int]$y){[void][ReleaseCapture]::SetCursorPos((40+$x),(40+$y));Msg 0x200}
 function Drag([int]$x,[int]$y,[int]$xx,[int]$yy){Move-ShowcasePointer $x $y;Msg 0x201;Move-ShowcasePointer $xx $yy;Msg 0x202}
 function Save([string]$name){
  Msg 0xF; Start-Sleep -Milliseconds 70
  $b=New-Object Drawing.Bitmap 1480,1100; $g=[Drawing.Graphics]::FromImage($b)
  try{$g.CopyFromScreen($form.Location,[Drawing.Point]::Empty,$form.ClientSize);$b.Save((Join-Path $outPath $name),[Drawing.Imaging.ImageFormat]::Png)}finally{$g.Dispose();$b.Dispose()}
 }
 Drag 64 100 1414 770
 Move-ShowcasePointer 145 353;Msg 0x201
 for($i=1;$i -le 120;$i++){Move-ShowcasePointer (145+$i*4) ([int](353+11*[Math]::Sin($i/9.0)));Start-Sleep -Milliseconds 2}
 Msg 0x202
 Move-ShowcasePointer 175 550;Msg 0x201
 for($i=1;$i -le 135;$i++){$t=$i/135.0*6.283;Move-ShowcasePointer ([int](320-145*[Math]::Cos($t))) ([int](550+51*[Math]::Sin($t)));Start-Sleep -Milliseconds 2}
 Msg 0x202
 Msg 0x100 0x32;Drag 600 594 700 455
 Msg 0x100 0x33;Drag 130 263 751 312
 Msg 0x100 0x34
 for($i=0;$i -lt 20;$i++){Move-ShowcasePointer (864+($i%5)*101) (396+[Math]::Floor($i/5)*78);Msg 0x201;Msg 0x202;Start-Sleep -Milliseconds 30}
 Msg 0x100 0x31;Move-ShowcasePointer 1435 1065;Save 'scene.png'
 foreach($tool in 1..4){Msg 0x100 (0x30+$tool);Move-ShowcasePointer 1435 1065;Save "tool-$tool.png"}
 # Hover over pen: actual contextual help and hover treatment.
 Move-ShowcasePointer 104 838;Start-Sleep -Milliseconds 120;Save 'hover-pen.png'
 Move-ShowcasePointer 1435 1065;Msg 0x100 0x1B
 [void][ReleaseCapture]::PostMessage($script:captureHwnd,0x10,[IntPtr]::Zero,[IntPtr]::Zero)
 if(!$process.WaitForExit(3000)){throw 'App did not close.'}
 [IO.File]::WriteAllText((Join-Path $outPath 'provenance.txt'),"exe=$([IO.Path]::GetFullPath($Exe))`ntheme=$Theme`nfixture=synthetic`ncanvas=1350x670 physical pixels`n")
 Write-Output "Saved native $Theme captures: $outPath"
} finally {
 if($null -ne $process -and !$process.HasExited){$process.Kill();$process.WaitForExit()}
 $env:LOCALAPPDATA=$oldLocal
 $form.Dispose();[Windows.Forms.Cursor]::Position=$oldPointer
 if(Test-Path $settings){Remove-Item -LiteralPath $settings -Recurse -Force}
}
