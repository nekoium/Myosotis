use myosotis_tools::{
    graphics::{Image, Runtime},
    metrics::{curve, export_difference, fidelity, ink, rounded},
    wide, Result,
};
use std::{
    fs::{self, File, OpenOptions},
    io::{BufWriter, Write},
    os::windows::io::AsRawHandle,
    path::{Path, PathBuf},
    process::{Child, Command},
    ptr::{null, null_mut},
    thread,
    time::{Duration, Instant},
};
use windows_sys::Win32::{
    Foundation::*,
    Graphics::Gdi::*,
    System::{DataExchange::*, Performance::*, ProcessStatus::*},
    UI::{
        Controls::*,
        HiDpi::*,
        Input::{KeyboardAndMouse::*, Pointer::*},
        WindowsAndMessaging::*,
    },
};
fn ms(start: Instant) -> f64 {
    start.elapsed().as_secs_f64() * 1000.0
}
fn pump(milliseconds: u64) {
    let start = Instant::now();
    while start.elapsed() < Duration::from_millis(milliseconds) {
        unsafe {
            let mut msg = MSG::default();
            while PeekMessageW(&mut msg, null_mut(), 0, 0, PM_REMOVE) != 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
        thread::sleep(Duration::from_millis(2));
    }
}
fn wincheck(value: i32, operation: &str) -> Result<()> {
    if value != 0 {
        Ok(())
    } else {
        Err(format!("{operation}: {}", std::io::Error::last_os_error()).into())
    }
}
struct Probe {
    window: HWND,
    left: i32,
    top: i32,
    width: i32,
    height: i32,
}
impl Probe {
    fn msg(&self, message: u32, value: i32) {
        unsafe {
            SendMessageW(self.window, message, value as usize, 0);
        }
    }
    fn move_to(&self, x: i32, y: i32) -> Result<()> {
        unsafe {
            wincheck(SetCursorPos(self.left + x, self.top + y), "SetCursorPos")?;
        }
        self.msg(0x200, 0);
        Ok(())
    }
    fn paint(&self) -> Result<()> {
        unsafe {
            wincheck(
                RedrawWindow(self.window, null(), null_mut(), 0x1 | 0x100 | 0x40),
                "RedrawWindow",
            )
        }
    }
    fn capture(&self) -> Result<()> {
        unsafe {
            wincheck(
                PostMessageW(self.window, 0x0312, 0x4d59, 0),
                "Capture hotkey",
            )?;
            for _ in 0..300 {
                if IsWindowVisible(self.window) != 0 {
                    break;
                }
                pump(10);
            }
            if IsWindowVisible(self.window) == 0 {
                return Err("Capture did not open".into());
            }
            SendMessageW(self.window, 0, 0, 0);
            SetForegroundWindow(self.window);
        }
        pump(150);
        self.move_to(0, 0)?;
        self.msg(0x204, 0);
        self.move_to(self.width, self.height)?;
        self.msg(0x205, 0);
        self.paint()
    }
    fn stroke(&self, index: i32) -> Result<()> {
        let span = self.width - 120;
        let row = self.height / 2 + 20 + (index % 12) * 6.max((self.height / 2 - 70) / 12);
        self.move_to(60, row)?;
        self.msg(0x201, 0);
        for i in 1..=80 {
            self.move_to(
                60 + span * i / 80,
                row + (12.0 * (i as f64 * 0.23 + index as f64).sin()) as i32,
            )?;
        }
        self.msg(0x202, 0);
        Ok(())
    }
    fn history(&self, n: i32) -> Result<()> {
        // Suppress display only during setup. Timed paints and input replay re-enable it.
        self.msg(0xb, 0);
        let result = (|| -> Result<()> {
            for i in 0..n {
                self.stroke(i)?;
            }
            Ok(())
        })();
        self.msg(0xb, 1);
        result?;
        self.paint()?;
        pump(100);
        Ok(())
    }
    fn mouse(&self, x: i32, y: i32, flags: u32) -> Result<()> {
        unsafe {
            let input = INPUT {
                r#type: INPUT_MOUSE,
                Anonymous: INPUT_0 {
                    mi: MOUSEINPUT {
                        dx: rounded(
                            (self.left + x - GetSystemMetrics(SM_XVIRTUALSCREEN)) as f64 * 65535.0
                                / (GetSystemMetrics(SM_CXVIRTUALSCREEN) - 1) as f64,
                        ),
                        dy: rounded(
                            (self.top + y - GetSystemMetrics(SM_YVIRTUALSCREEN)) as f64 * 65535.0
                                / (GetSystemMetrics(SM_CYVIRTUALSCREEN) - 1) as f64,
                        ),
                        dwFlags: flags | 0x8000 | 0x4000,
                        ..Default::default()
                    },
                },
            };
            if SendInput(1, &input, std::mem::size_of::<INPUT>() as i32) != 1 {
                return Err("SendInput failed".into());
            }
        }
        Ok(())
    }
    fn foreground(&self) -> bool {
        unsafe { IsWindowVisible(self.window) != 0 && GetForegroundWindow() == self.window }
    }
    fn screenshot(&self) -> Result<Image> {
        Image::capture(self.left, self.top, self.width, self.height)
    }
    fn real_curve(
        &self,
        folder: &Path,
        n: i32,
        replay: &mut impl Write,
        seconds: i32,
    ) -> Result<()> {
        let count = 240 * seconds;
        self.mouse(50, self.height / 4, 1)?;
        pump(40);
        self.mouse(50, self.height / 4, 2)?;
        thread::sleep(Duration::from_millis(12));
        let result = (|| -> Result<f64> {
            let clock = Instant::now();
            let mut worst: f64 = 0.;
            for i in 0..=count {
                let due = i as f64 * 1000.0 * seconds as f64 / count as f64;
                while ms(clock) < due {
                    if due - ms(clock) > 1.0 {
                        thread::yield_now();
                    } else {
                        std::hint::spin_loop();
                    }
                }
                if !self.foreground() {
                    return Err("Replay lost its visible foreground window; run invalid".into());
                }
                if i > 0 {
                    let mut observed = POINT::default();
                    unsafe {
                        wincheck(GetCursorPos(&mut observed), "GetCursorPos")?;
                    }
                    let (px, py) = curve(self.width, self.height, seconds, i - 1);
                    if (observed.x - self.left - px).abs() > 2
                        || (observed.y - self.top - py).abs() > 2
                    {
                        return Err("External pointer movement during replay; run invalid".into());
                    }
                }
                let (x, y) = curve(self.width, self.height, seconds, i);
                let actual = ms(clock);
                worst = worst.max(actual - due);
                self.mouse(x, y, 1)?;
                writeln!(replay, "{n},{i},{actual:.3},{x},{y}")?;
            }
            if !self.foreground() {
                return Err("Replay lost focus or capture; run invalid".into());
            }
            Ok(worst)
        })();
        // Release our button even when an interrupted replay is rejected.
        let released = self.mouse(self.width - 50, self.height / 4, 4);
        let worst = result?;
        released?;
        pump(700);
        self.paint()?;
        pump(100);
        let image = self.screenshot()?;
        image.save(&folder.join(format!("curve-{n}.png")))?;
        let (median, p95, missing, samples) =
            fidelity(&image.pixels()?, self.width, self.height, seconds);
        writeln!(
            OpenOptions::new()
                .append(true)
                .open(folder.join("fidelity.csv"))?,
            "{n},{median:.3},{p95:.3},{missing},{samples},{worst:.3}"
        )?;
        Ok(())
    }
    fn history_costs(&self, folder: &Path) -> Result<()> {
        self.history(250)?;
        let mut out = BufWriter::new(File::create(folder.join("history-costs.csv"))?);
        writeln!(out, "action,dispatch_ms,paint_ms")?;
        for key in [0x5a, 0x59, 0x52, 0x5a, 0x0d] {
            let start = Instant::now();
            self.msg(0x100, key);
            let input = ms(start);
            let paint = Instant::now();
            if unsafe { IsWindowVisible(self.window) != 0 } {
                self.paint()?;
            }
            writeln!(out, "{key},{input:.3},{:.3}", ms(paint))?;
        }
        out.flush()?;
        Ok(())
    }
    fn begin(&self, x: i32, y: i32) -> Result<()> {
        self.move_to(x, y)?;
        self.msg(0x201, 0);
        Ok(())
    }
    fn shapes(&self, folder: &Path) -> Result<()> {
        self.begin(90, 90)?;
        self.move_to(240, 90)?;
        self.move_to(240, 180)?;
        self.move_to(390, 180)?;
        self.msg(0x202, 0);
        self.begin(90, 260)?;
        self.move_to(500, 260)?;
        self.msg(0x202, 0); // Two-event flick.
        for j in 0..5 {
            self.begin(90 + j * 50, 340)?;
            self.msg(0x202, 0);
            pump(270);
        }
        self.begin(700, 120)?;
        for i in 1..=240 {
            self.move_to(
                700 + (100.0 * (i as f64 * 0.08).sin()) as i32,
                120 + (65.0 * (i as f64 * 0.16).sin()) as i32,
            )?;
        }
        self.msg(0x202, 0);
        self.begin(950, 300)?;
        self.move_to(1100, 410)?;
        self.move_to(950, 410)?;
        self.move_to(1100, 300)?;
        self.msg(0x202, 0);
        // Space uses the same stroke path and width code.
        self.move_to(90, 430)?;
        self.msg(0x100, 32);
        for i in 1..=60 {
            self.move_to(90 + i * 6, 430 + (22.0 * (i as f64 * 0.2).sin()) as i32)?;
        }
        self.msg(0x101, 32);
        self.paint()?;
        pump(100);
        self.screenshot()?.save(&folder.join("shapes.png"))?;
        let device = Pen::new()?;
        for row in 0..3 {
            for i in 0..=100 {
                let pressure = if row == 0 {
                    100
                } else if row == 1 {
                    900
                } else {
                    100 + i * 8
                };
                let info = POINTER_TYPE_INFO {
                    r#type: 3,
                    Anonymous: POINTER_TYPE_INFO_0 {
                        penInfo: POINTER_PEN_INFO {
                            pointerInfo: POINTER_INFO {
                                pointerType: 3,
                                pointerId: 1,
                                pointerFlags: if i == 0 {
                                    0x10000 | 2 | 4
                                } else if i == 100 {
                                    0x40000 | 2
                                } else {
                                    0x20000 | 2 | 4
                                },
                                ptPixelLocation: POINT {
                                    x: self.left + 100 + i * 9,
                                    y: self.top + 540 + row * 70,
                                },
                                ..Default::default()
                            },
                            penMask: 1,
                            pressure: pressure as u32,
                            ..Default::default()
                        },
                    },
                };
                unsafe {
                    wincheck(
                        InjectSyntheticPointerInput(device.0, &info, 1),
                        "Pen injection",
                    )?;
                }
                pump(5);
            }
            pump(300);
        }
        drop(device);
        self.paint()?;
        pump(100);
        let bitmap = self.screenshot()?;
        bitmap.save(&folder.join("pressure.png"))?;
        let pixels = bitmap.pixels()?;
        let mut widths = [0; 3];
        for row in 0..3 {
            for y in 520 + row * 70..560 + row * 70 {
                if ink(pixels[(y * self.width + 550) as usize]) {
                    widths[row as usize] += 1;
                }
            }
        }
        let widths_text = format!("{},{},{}", widths[0], widths[1], widths[2]);
        fs::write(folder.join("pressure-widths.txt"), &widths_text)?;
        if widths[0] < 1 || widths[1] <= widths[0] {
            return Err(
                format!("Synthetic pressure did not change ink width: {widths_text}").into(),
            );
        }
        let start = Instant::now();
        self.msg(0x100, 13);
        let copy = ms(start);
        let exported = clipboard_image()?;
        if exported.width != self.width || exported.height != self.height {
            return Err("Export size mismatch".into());
        }
        exported.save(&folder.join("export.png"))?;
        let (different, mask, missing) =
            export_difference(&pixels, &exported.pixels()?, self.width, self.height);
        fs::write(folder.join("export-comparison.txt"),format!("copy_ms={copy:.3}\nexact_pixel_differences={different}\nmask_differences={mask}\nmask_differences_beyond_1px={missing}\n"))?;
        if missing > 0 {
            return Err(
                format!("Preview/export ink differs beyond antialias edges: {missing}").into(),
            );
        }
        println!("Shapes, synthetic pressure, and preview/export comparison passed");
        Ok(())
    }
    fn long_point(&self, i: i32) -> Result<()> {
        self.move_to(
            self.width / 2 + (self.width as f64 * 0.3 * (i as f64 * 0.07).sin()) as i32,
            self.height / 2 + (self.height as f64 * 0.3 * (i as f64 * 0.09).sin()) as i32,
        )
    }
    fn long_stroke(&self, folder: &Path) -> Result<()> {
        self.long_point(0)?;
        self.msg(0x201, 0);
        let mut out = BufWriter::new(File::create(folder.join("active.csv"))?);
        writeln!(out, "samples,frame,input_ms,paint_ms")?;
        let mut previous = 0;
        for limit in [80, 320, 640, 5000, 20000] {
            self.msg(0xb, 0);
            let result = (|| -> Result<()> {
                for i in previous + 1..=limit {
                    self.long_point(i)?;
                }
                Ok(())
            })();
            self.msg(0xb, 1);
            result?;
            for frame in 0..6 {
                let start = Instant::now();
                self.long_point(limit + frame + 1)?;
                let input = ms(start);
                let paint = Instant::now();
                self.paint()?;
                writeln!(
                    out,
                    "{},{frame},{input:.3},{:.3}",
                    limit + frame + 1,
                    ms(paint)
                )?;
            }
            out.flush()?;
            previous = limit + 6;
            println!("active samples={limit} completed");
        }
        self.msg(0x202, 0);
        Ok(())
    }
    fn evaluate(
        &self,
        child: &Child,
        folder: &Path,
        densities: &[i32],
        frames: i32,
        seconds: i32,
    ) -> Result<()> {
        let mut output = BufWriter::new(File::create(folder.join("frames.csv"))?);
        let mut replay = BufWriter::new(File::create(folder.join("input.csv"))?);
        writeln!(
            output,
            "history_strokes,frame,input_ms,paint_ms,private_bytes"
        )?;
        writeln!(replay, "history_strokes,sample,time_ms,x,y")?;
        for &n in densities {
            self.capture()?;
            if n == -4 {
                self.history_costs(folder)?;
            } else if n == -2 {
                self.shapes(folder)?;
            } else if n < 0 {
                self.long_stroke(folder)?;
            } else {
                self.history(n)?;
                let mut idle =
                    BufWriter::new(File::create(folder.join(format!("static-{n}.csv")))?);
                writeln!(idle, "frame,paint_ms")?;
                for frame in 0..8 {
                    let start = Instant::now();
                    self.paint()?;
                    writeln!(idle, "{frame},{:.3}", ms(start))?;
                }
                idle.flush()?;
                self.begin(50, self.height / 4)?;
                for i in 0..frames + 5 {
                    let x = 70 + i * (self.width - 140) / (frames + 5);
                    let y = self.height / 4 + (30.0 * (i as f64 * 0.3).sin()) as i32;
                    let start = Instant::now();
                    self.move_to(x, y)?;
                    let input = ms(start);
                    let paint = Instant::now();
                    self.paint()?;
                    let elapsed = ms(paint);
                    if i >= 5 {
                        writeln!(
                            output,
                            "{n},{},{input:.3},{elapsed:.3},{}",
                            i - 5,
                            private_bytes(child)?
                        )?;
                    }
                }
                self.msg(0x202, 0);
                self.msg(0x100, 0x5a); // Remove the measurement stroke before fidelity replay.
                self.real_curve(folder, n, &mut replay, seconds)?;
                output.flush()?;
                replay.flush()?;
                println!("history={n} completed");
            }
            self.msg(0x100, 27);
            pump(100);
        }
        output.flush()?;
        replay.flush()?;
        Ok(())
    }
}
struct Pen(HSYNTHETICPOINTERDEVICE);
impl Pen {
    fn new() -> Result<Self> {
        let handle = unsafe { CreateSyntheticPointerDevice(3, 1, 3) };
        if handle.is_null() {
            Err(std::io::Error::last_os_error().into())
        } else {
            Ok(Self(handle))
        }
    }
}
impl Drop for Pen {
    fn drop(&mut self) {
        unsafe {
            DestroySyntheticPointerDevice(self.0);
        }
    }
}
fn private_bytes(child: &Child) -> Result<usize> {
    let mut counters = PROCESS_MEMORY_COUNTERS_EX::default();
    counters.cb = std::mem::size_of_val(&counters) as u32;
    unsafe {
        wincheck(
            GetProcessMemoryInfo(
                child.as_raw_handle(),
                &mut counters as *mut _ as *mut PROCESS_MEMORY_COUNTERS,
                counters.cb,
            ),
            "GetProcessMemoryInfo",
        )?;
    }
    Ok(counters.PrivateUsage)
}
fn clipboard_image() -> Result<Image> {
    unsafe {
        wincheck(OpenClipboard(null_mut()), "OpenClipboard")?;
        // Windows synthesizes CF_BITMAP from CF_DIB; copy it before closing the clipboard.
        let bitmap = GetClipboardData(2);
        let result = if bitmap.is_null() {
            Err("Clipboard has no bitmap".into())
        } else {
            Image::from_bitmap(bitmap)
        };
        CloseClipboard();
        result
    }
}
struct Desktop {
    fixture: HWND,
    pointer: POINT,
    previous: HWND,
    child: Option<Child>,
    window: HWND,
}
impl Drop for Desktop {
    fn drop(&mut self) {
        if let Some(child) = self.child.as_mut() {
            if matches!(child.try_wait(), Ok(None)) {
                unsafe {
                    if !self.window.is_null() {
                        if IsWindowVisible(self.window) != 0 {
                            SendMessageW(self.window, 0x100, 27, 0);
                        }
                        SendMessageW(self.window, 0x10, 0, 0);
                    }
                }
                let start = Instant::now();
                while matches!(child.try_wait(), Ok(None))
                    && start.elapsed() < Duration::from_secs(3)
                {
                    pump(10);
                }
                if matches!(child.try_wait(), Ok(None)) {
                    let _ = child.kill();
                }
                let _ = child.wait();
            }
        }
        unsafe {
            if !self.fixture.is_null() {
                DestroyWindow(self.fixture);
            }
            SetCursorPos(self.pointer.x, self.pointer.y);
            if !self.previous.is_null() {
                SetForegroundWindow(self.previous);
            }
        }
    }
}
fn run() -> Result<()> {
    let mut args = std::env::args().skip(1);
    if args.next().as_deref() != Some("--interactive") {
        return Err("Native replay requires exclusive cursor and focus control for the entire run. Pass --interactive only when the desktop is available.".into());
    }
    let exe = PathBuf::from(args.next().ok_or("Missing executable")?);
    let folder = PathBuf::from(args.next().ok_or("Missing result folder")?);
    let densities: Vec<i32> = args
        .next()
        .ok_or("Missing densities")?
        .split(',')
        .map(str::parse)
        .collect::<std::result::Result<_, _>>()?;
    let frames: i32 = args.next().ok_or("Missing frames")?.parse()?;
    let seconds: i32 = args.next().ok_or("Missing seconds")?.parse()?;
    if args.next().is_some()
        || densities.is_empty()
        || !(1..=60).contains(&seconds)
        || !(1..=100000).contains(&frames)
        || densities.iter().any(|n| !(-4..=100000).contains(n))
    {
        return Err("Invalid diagnostic arguments".into());
    }
    if !exe.is_file() {
        return Err("Executable does not exist".into());
    }
    let class = wide("Myosotis.Native.Overlay");
    unsafe {
        if !FindWindowW(class.as_ptr(), null()).is_null() {
            return Err(
                "Quit the existing Myosotis before diagnostics; no process was changed.".into(),
            );
        }
    }
    let _runtime = Runtime::new()?;
    unsafe {
        wincheck(
            SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2),
            "SetProcessDpiAwarenessContext",
        )?;
        let mut desktop = Desktop {
            fixture: null_mut(),
            pointer: POINT::default(),
            previous: GetForegroundWindow(),
            child: None,
            window: null_mut(),
        };
        wincheck(GetCursorPos(&mut desktop.pointer), "GetCursorPos")?;
        let (screen_w, screen_h) = (GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN));
        let (width, height) = (1400.min(screen_w - 160), 850.min(screen_h - 220));
        if width < 700 || height < 400 {
            return Err("Display too small for the diagnostic canvas".into());
        }
        // Built-in STATIC's white rectangle paints the same opaque primary-screen fixture as the former Form.
        desktop.fixture = CreateWindowExW(
            WS_EX_TOOLWINDOW,
            wide("STATIC").as_ptr(),
            wide("Myosotis performance test — white fixture").as_ptr(),
            WS_POPUP | WS_VISIBLE | 6,
            0,
            0,
            screen_w,
            screen_h,
            null_mut(),
            null_mut(),
            null_mut(),
            null(),
        );
        if desktop.fixture.is_null() {
            return Err(std::io::Error::last_os_error().into());
        }
        SetForegroundWindow(desktop.fixture);
        pump(200);
        desktop.child = Some(Command::new(&exe).spawn()?);
        for _ in 0..300 {
            desktop.window = FindWindowW(class.as_ptr(), null());
            if !desktop.window.is_null() {
                break;
            }
            pump(10);
        }
        if desktop.window.is_null() || desktop.child.as_mut().unwrap().try_wait()?.is_some() {
            return Err("Test app failed to start".into());
        }
        let probe = Probe {
            window: desktop.window,
            left: 80,
            top: 80,
            width,
            height,
        };
        let mut frequency = 0;
        QueryPerformanceFrequency(&mut frequency);
        fs::write(folder.join("environment.txt"),format!("exe={}\ncanvas={width}x{height}\nscreen={{X=0,Y=0,Width={screen_w},Height={screen_h}}}\ndpi={}\nclock_hz={frequency}\n",exe.display(),GetDpiForWindow(probe.window)))?;
        fs::write(folder.join("fidelity.csv"),"history_strokes,median_distance_px,p95_distance_px,missing_over_3px,samples,max_inject_lateness_ms\n")?;
        probe.evaluate(
            desktop.child.as_ref().unwrap(),
            &folder,
            &densities,
            frames,
            seconds,
        )?;
    }
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
