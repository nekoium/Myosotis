mod capture;
#[cfg(test)]
mod completion_tests;
mod render;
mod startup;

use crate::{
    document::{hit_edge, resize, Document, Point, Rect, Tool},
    settings::{self, Theme},
};
use ::windows::{
    core::*,
    Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        System::{Com::*, LibraryLoader::*},
        UI::{HiDpi::*, Input::KeyboardAndMouse::*, Shell::*, WindowsAndMessaging::*},
    },
};
use std::{
    cell::RefCell,
    mem::size_of,
    time::{Duration, Instant},
};

const DOUBLE_CLICK_TIME: Duration = Duration::from_millis(250);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Input {
    Mouse,
    Space,
}
#[derive(Clone, Copy)]
struct Click {
    point: Point,
    since: Instant,
    input: Input,
}
impl Click {
    fn matches(self, point: Point, input: Input, scale: f32) -> bool {
        self.input == input
            && self.since.elapsed() <= DOUBLE_CLICK_TIME
            && (point.x - self.point.x).abs() + (point.y - self.point.y).abs() < 6. * scale
    }
}

const TRAY: u32 = WM_APP + 1;
const HOTKEY: i32 = 0x4D59;
const CAPTURE: usize = 1;
const QUIT: usize = 2;
const THEME_DARK: usize = 3;
const THEME_LIGHT: usize = 4;
const START_AT_LOGIN: usize = 5;
const HINT: &str = "Double-click or Ctrl to copy · Z undo · Esc cancel";
thread_local! { static APP: RefCell<Option<App>> = const { RefCell::new(None) }; }

#[derive(Clone, Copy)]
enum Gesture {
    Select { start: Point, blank: bool },
    Draw,
    Resize { original: Rect, edge: u8 },
}
struct App {
    hwnd: HWND,
    previous: HWND,
    doc: Document,
    renderer: Option<render::Renderer>,
    theme: Theme,
    pixels: Vec<u8>,
    origin: POINT,
    full: Rect,
    scale: f32,
    visible: bool,
    pointer: Point,
    gesture: Option<Gesture>,
    click_start: Option<Click>,
    pending_click: Option<Click>,
    space: bool,
    hover: Option<(u8, Instant)>,
    hint: String,
    tray: NOTIFYICONDATAW,
    taskbar_created: u32,
}
impl App {
    unsafe fn activate(&mut self) -> Result<()> {
        if self.visible {
            return Ok(());
        }
        self.previous = GetForegroundWindow();
        self.origin = POINT {
            x: GetSystemMetrics(SM_XVIRTUALSCREEN),
            y: GetSystemMetrics(SM_YVIRTUALSCREEN),
        };
        let width = GetSystemMetrics(SM_CXVIRTUALSCREEN);
        let height = GetSystemMetrics(SM_CYVIRTUALSCREEN);
        let started = Instant::now();
        self.pixels = capture::Surface::capture(self.origin.x, self.origin.y, width, height)?;
        self.full = Rect::between(Point::default(), Point::new(width as f32, height as f32));
        self.pointer = self.cursor();
        let monitor = MonitorFromPoint(
            POINT {
                x: self.pointer.x as i32 + self.origin.x,
                y: self.pointer.y as i32 + self.origin.y,
            },
            MONITOR_DEFAULTTONEAREST,
        );
        let (mut x, mut y) = (96, 96);
        let _ = GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut x, &mut y);
        self.scale = x as f32 / 96.;
        SetWindowPos(
            self.hwnd,
            HWND_TOPMOST,
            self.origin.x,
            self.origin.y,
            width,
            height,
            SWP_NOACTIVATE,
        )?;
        self.renderer = Some(render::Renderer::new(
            self.hwnd,
            width as u32,
            height as u32,
            &self.pixels,
            self.scale,
            self.theme,
        )?);
        self.doc = Document::default();
        self.gesture = None;
        self.click_start = None;
        self.pending_click = None;
        self.hover = None;
        self.space = false;
        self.hint.clear();
        self.visible = true;
        let _ = ShowWindow(self.hwnd, SW_SHOW);
        let _ = SetForegroundWindow(self.hwnd);
        let _ = SetFocus(self.hwnd);
        SetTimer(self.hwnd, 1, 30, None);
        self.repaint();
        eprintln!("Capture ready in {} ms", started.elapsed().as_millis());
        Ok(())
    }
    unsafe fn close_overlay(&mut self) {
        self.visible = false;
        self.click_start = None;
        self.pending_click = None;
        self.gesture = None;
        self.space = false;
        self.doc = Document::default();
        let _ = ReleaseCapture();
        let _ = KillTimer(self.hwnd, 1);
        // Restore while we still own foreground activation rights; hiding first can transfer them.
        if !self.previous.is_invalid() && IsWindow(self.previous).as_bool() {
            let _ = SetForegroundWindow(self.previous);
        }
        let _ = ShowWindow(self.hwnd, SW_HIDE);
        self.renderer = None;
        self.pixels.clear();
        self.pixels.shrink_to_fit();
    }
    fn flush_click(&mut self) {
        if self.pending_click.take().is_some() {
            self.doc.finish();
        }
    }
    fn discard_click(&mut self) {
        if self.pending_click.take().is_some() {
            // Keep the first click visible but uncommitted until recognition resolves.
            // Dropping it preserves both earlier annotations and the redo history.
            self.doc.active = None;
        }
    }
    unsafe fn copy(&mut self) {
        self.flush_click();
        if self.doc.selection.is_none() {
            return;
        }
        self.doc.finish();
        let started = Instant::now();
        let result = self
            .renderer
            .as_ref()
            .unwrap()
            .export(&self.doc, &self.pixels, self.full)
            .and_then(|surface| surface.to_clipboard(self.hwnd));
        match result {
            Ok(()) => {
                eprintln!("Clipboard complete in {} ms", started.elapsed().as_millis());
                self.close_overlay();
            }
            Err(error) => {
                eprintln!("Clipboard copy failed: {error}");
                self.hint = format!("Copy failed; image kept. Press Enter to retry. {error}");
                self.repaint();
            }
        }
    }
    unsafe fn cursor(&self) -> Point {
        let mut p = POINT::default();
        let _ = GetCursorPos(&mut p);
        Point::new((p.x - self.origin.x) as f32, (p.y - self.origin.y) as f32)
    }
    unsafe fn monitor(&self, p: Point) -> Rect {
        let monitor = MonitorFromPoint(
            POINT {
                x: self.origin.x + p.x as i32,
                y: self.origin.y + p.y as i32,
            },
            MONITOR_DEFAULTTONEAREST,
        );
        let mut info = MONITORINFO {
            cbSize: size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if !GetMonitorInfoW(monitor, &mut info).as_bool() {
            return self.full;
        }
        Rect {
            left: (info.rcMonitor.left - self.origin.x) as f32,
            top: (info.rcMonitor.top - self.origin.y) as f32,
            right: (info.rcMonitor.right - self.origin.x) as f32,
            bottom: (info.rcMonitor.bottom - self.origin.y) as f32,
        }
    }
    unsafe fn toolbar(&self) -> Option<Rect> {
        let selection = self.doc.selection?;
        let monitor = self.monitor(Point::new(
            (selection.left + selection.right) / 2.,
            (selection.top + selection.bottom) / 2.,
        ));
        let width = render::TOOLBAR_CELL * 6. * self.scale;
        let height = render::TOOLBAR_HEIGHT * self.scale;
        let gap = 12. * self.scale;
        let x = selection.left.clamp(
            monitor.left + gap,
            (monitor.right - width - gap).max(monitor.left + gap),
        );
        let y = if selection.bottom + height + gap + 45. * self.scale <= monitor.bottom {
            selection.bottom + gap
        } else if selection.top - height - gap >= monitor.top {
            selection.top - height - gap
        } else {
            monitor.top + gap
        };
        Some(Rect {
            left: x,
            top: y,
            right: x + width,
            bottom: y + height,
        })
    }
    unsafe fn down(&mut self, blank: bool, input: Input) {
        if !self.visible || self.gesture.is_some() || self.space {
            return;
        }
        let p = self.cursor();
        self.pointer = p;
        let toolbar = self.toolbar().is_some_and(|bar| bar.contains(p));
        let resizing = self.doc.selection.is_some_and(|selection| {
            self.hover.is_some_and(|(edge, since)| {
                since.elapsed().as_millis() >= 350
                    && hit_edge(selection, p, 6. * self.scale) == edge
            })
        });
        if !blank
            && !toolbar
            && !resizing
            && self
                .pending_click
                .is_some_and(|click| click.matches(p, input, self.scale))
        {
            self.discard_click();
            self.copy();
            return;
        }
        self.flush_click();
        self.click_start = None;
        if self.doc.selection.is_none() {
            self.gesture = Some(Gesture::Select { start: p, blank });
            if !blank {
                self.click_start = Some(Click {
                    point: p,
                    since: Instant::now(),
                    input,
                });
            }
        } else {
            if blank {
                self.doc.blank = !self.doc.blank;
                self.repaint();
                return;
            }
            if let Some(bar) = self.toolbar() {
                if bar.contains(p) {
                    let item = ((p.x - bar.left) / (render::TOOLBAR_CELL * self.scale)) as usize;
                    match item {
                        0..=3 => self.doc.tool = Tool::ALL[item],
                        4 => self.copy(),
                        5 => self.close_overlay(),
                        _ => {}
                    }
                    self.repaint();
                    return;
                }
            }
            if resizing {
                self.gesture = Some(Gesture::Resize {
                    original: self.doc.selection.unwrap(),
                    edge: self.hover.unwrap().0,
                });
            } else {
                self.doc.begin(p, self.scale);
                // Outside clicks still participate in completion, but create no ink.
                self.gesture = Some(Gesture::Draw);
                self.click_start = Some(Click {
                    point: p,
                    since: Instant::now(),
                    input,
                });
            }
        }
        if self.gesture.is_some() {
            SetCapture(self.hwnd);
            self.space = input == Input::Space;
        }
        self.repaint();
    }
    unsafe fn up(&mut self, blank: bool, input: Input) {
        if self.space != (input == Input::Space) {
            return;
        }
        let p = self.cursor();
        match self.gesture {
            Some(Gesture::Select {
                start,
                blank: expected,
            }) if expected == blank => {
                let distance = (p.x - start.x).abs() + (p.y - start.y).abs();
                let selection = if distance < 6. * self.scale {
                    self.monitor(p)
                } else {
                    Rect::between(start, p).clamp(self.full)
                };
                if selection.width() >= 1. && selection.height() >= 1. {
                    self.doc.select(selection, blank);
                }
            }
            Some(Gesture::Draw) if !blank => {
                self.doc.update(p);
            }
            Some(Gesture::Resize { original, edge }) if !blank => {
                self.doc.selection = Some(resize(original, edge, p, self.full, 1.))
            }
            _ => return,
        }
        self.pending_click = self
            .click_start
            .take()
            .filter(|click| click.matches(p, input, self.scale));
        if self.pending_click.is_none() {
            self.doc.finish();
        }
        self.space = false;
        self.gesture = None;
        let _ = ReleaseCapture();
        self.repaint();
    }
    unsafe fn moved(&mut self) {
        let p = self.cursor();
        self.pointer = p;
        if self.click_start.is_some_and(|click| {
            (p.x - click.point.x).abs() + (p.y - click.point.y).abs() >= 6. * self.scale
        }) {
            self.click_start = None;
        }
        match self.gesture {
            Some(Gesture::Draw) => self.doc.update(p),
            Some(Gesture::Resize { original, edge }) => {
                self.doc.selection = Some(resize(original, edge, p, self.full, 1.))
            }
            Some(Gesture::Select { .. }) => {}
            None if self.space => self.doc.update(p),
            None => {
                let edge = self
                    .doc
                    .selection
                    .map_or(0, |r| hit_edge(r, p, 6. * self.scale));
                if edge == 0 {
                    self.hover = None;
                } else if self.hover.is_none_or(|(previous, _)| previous != edge) {
                    self.hover = Some((edge, Instant::now()));
                }
            }
        }
        self.set_cursor();
        self.repaint();
    }
    unsafe fn set_cursor(&self) {
        let edge = match self.gesture {
            Some(Gesture::Resize { edge, .. }) => edge,
            _ => self
                .hover
                .filter(|(_, since)| since.elapsed().as_millis() >= 350)
                .map_or(0, |(e, _)| e),
        };
        let id = match edge {
            1 | 4 => IDC_SIZEWE,
            2 | 8 => IDC_SIZENS,
            3 | 12 => IDC_SIZENWSE,
            6 | 9 => IDC_SIZENESW,
            _ => IDC_CROSS,
        };
        let id = if self.toolbar().is_some_and(|r| r.contains(self.pointer)) {
            IDC_HAND
        } else {
            id
        };
        if let Ok(cursor) = LoadCursorW(None, id) {
            SetCursor(cursor);
        }
    }
    unsafe fn key(&mut self, key: usize, repeat: bool) {
        if key == 0x20 {
            if !repeat {
                self.down(false, Input::Space);
            }
            return;
        }
        if key != 0x1B {
            self.flush_click();
            self.click_start = None;
        }
        match key {
            0x1B => self.close_overlay(),
            0x0D | 0x11 if !repeat => self.copy(),
            0x31..=0x34 if !repeat => {
                self.doc.finish();
                self.doc.tool = Tool::ALL[key - 0x31];
            }
            0x5A if !repeat => {
                self.gesture = None;
                self.space = false;
                self.doc.undo();
                let _ = ReleaseCapture();
            }
            0x59 if !repeat => self.doc.redo(),
            0x52 if !repeat => {
                self.gesture = None;
                self.space = false;
                self.doc.reset();
                let _ = ReleaseCapture();
            }
            _ => {}
        }
        self.repaint();
    }
    unsafe fn repaint(&self) {
        if self.visible {
            let _ = InvalidateRect(self.hwnd, None, false);
        }
    }
    unsafe fn paint(&mut self) {
        let mut ps = PAINTSTRUCT::default();
        BeginPaint(self.hwnd, &mut ps);
        if self.visible {
            let selecting = if let Some(Gesture::Select { start, .. }) = self.gesture {
                Some(Rect::between(start, self.pointer))
            } else {
                None
            };
            let toolbar_label = self.toolbar().and_then(|bar| {
                bar.contains(self.pointer).then(|| {
                    let item = ((self.pointer.x - bar.left) / (render::TOOLBAR_CELL * self.scale))
                        as usize;
                    render::TOOLBAR_LABELS.get(item).copied().unwrap_or("")
                })
            });
            let hint = if !self.hint.is_empty() {
                self.hint.as_str()
            } else if let Some(label) = toolbar_label {
                label
            } else if self.doc.selection.is_some() {
                HINT
            } else {
                "Myosotis · Drag to capture · Right-drag for white canvas · Click for monitor · Esc cancel"
            };
            let result = self.renderer.as_ref().map(|r| {
                r.paint(
                    &self.doc,
                    self.full,
                    selecting,
                    self.toolbar(),
                    self.pointer,
                    hint,
                )
            });
            if let Some(Err(error)) = result {
                self.hint = format!("Display error: {error}. Escape and capture again.");
            }
        }
        let _ = EndPaint(self.hwnd, &ps);
    }
    unsafe fn change_theme(&mut self, theme: Theme) {
        if self.theme == theme {
            return;
        }
        self.theme = theme;
        if let Some(renderer) = &mut self.renderer {
            renderer.theme = theme;
        }
        self.repaint();
        if let Err(error) = settings::path().and_then(|path| theme.save(&path)) {
            show_error(&format!(
                "The theme changed for this session, but could not be saved.\n\n{error}\n\nCheck %LOCALAPPDATA%\\Myosotis\\settings.toml and its folder permissions."
            ));
        }
    }
    unsafe fn tray_menu(&mut self) {
        if let Ok(menu) = CreatePopupMenu() {
            let _ = AppendMenuW(
                menu,
                MF_STRING,
                CAPTURE,
                w!("Capture / blank canvas    Ctrl+Alt+S"),
            );
            let _ = AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null());
            if let Ok(themes) = CreatePopupMenu() {
                let _ = AppendMenuW(themes, MF_STRING, THEME_DARK, w!("Dark"));
                let _ = AppendMenuW(themes, MF_STRING, THEME_LIGHT, w!("Light"));
                let selected = match self.theme {
                    Theme::Dark => THEME_DARK,
                    Theme::Light => THEME_LIGHT,
                };
                let _ = CheckMenuRadioItem(
                    themes,
                    THEME_DARK as u32,
                    THEME_LIGHT as u32,
                    selected as u32,
                    MF_BYCOMMAND.0,
                );
                if AppendMenuW(menu, MF_POPUP, themes.0 as usize, w!("Theme")).is_err() {
                    let _ = DestroyMenu(themes);
                }
            }
            let startup_state = startup::enabled();
            let (startup_flags, startup_label) = match &startup_state {
                Ok(true) if startup::disabled_in_windows() => (
                    MF_STRING | MF_CHECKED,
                    w!("Start at login (Disabled in Windows)"),
                ),
                Ok(true) => (MF_STRING | MF_CHECKED, w!("Start at login")),
                Ok(false) => (MF_STRING, w!("Start at login")),
                Err(_) => (
                    MF_STRING | MF_GRAYED,
                    w!("Start at login (state unavailable)"),
                ),
            };
            let _ = AppendMenuW(menu, startup_flags, START_AT_LOGIN, startup_label);
            let _ = AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null());
            let _ = AppendMenuW(menu, MF_STRING, QUIT, w!("Quit Myosotis"));
            let mut p = POINT::default();
            let _ = GetCursorPos(&mut p);
            let _ = SetForegroundWindow(self.hwnd);
            let action = TrackPopupMenu(
                menu,
                TPM_RETURNCMD | TPM_RIGHTBUTTON,
                p.x,
                p.y,
                0,
                self.hwnd,
                None,
            )
            .0 as usize;
            let _ = DestroyMenu(menu);
            let _ = PostMessageW(self.hwnd, WM_NULL, WPARAM(0), LPARAM(0));
            match action {
                CAPTURE => {
                    if let Err(e) = self.activate() {
                        show_error(&e.to_string());
                    }
                }
                THEME_DARK => self.change_theme(Theme::Dark),
                THEME_LIGHT => self.change_theme(Theme::Light),
                START_AT_LOGIN => {
                    // Recheck after the menu closes in case another process changed the entry.
                    if let Err(error) =
                        startup::enabled().and_then(|enabled| startup::set_enabled(!enabled))
                    {
                        show_error(&format!(
                            "Could not change Start at login.\n\n{error}\n\nCheck your Windows startup permissions. Reopen the tray menu to see the current registration."
                        ));
                    }
                }
                QUIT => {
                    let _ = DestroyWindow(self.hwnd);
                }
                _ => {}
            }
            if let Err(error) = startup_state {
                show_error(&format!(
                    "Could not read Start at login. No startup registration was changed.\n\n{error}\n\nCheck your Windows startup permissions."
                ));
            }
        }
    }
    unsafe fn message(&mut self, msg: u32, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        match msg {
            WM_HOTKEY if wparam.0 == HOTKEY as usize => {
                if let Err(e) = self.activate() {
                    show_error(&format!("Capture failed.\n\n{e}"));
                }
            }
            WM_PAINT => self.paint(),
            WM_ERASEBKGND => return Some(LRESULT(1)),
            WM_LBUTTONDOWN => self.down(false, Input::Mouse),
            WM_RBUTTONDOWN => self.down(true, Input::Mouse),
            WM_LBUTTONUP => self.up(false, Input::Mouse),
            WM_RBUTTONUP => self.up(true, Input::Mouse),
            WM_MOUSEMOVE if self.visible => self.moved(),
            WM_TIMER if self.visible => {
                if self
                    .pending_click
                    .is_some_and(|click| click.since.elapsed() > DOUBLE_CLICK_TIME)
                {
                    self.flush_click();
                }
                self.set_cursor();
            }
            WM_SETCURSOR if self.visible => {
                self.set_cursor();
                return Some(LRESULT(1));
            }
            WM_KEYDOWN if self.visible => self.key(wparam.0, lparam.0 & (1 << 30) != 0),
            WM_KEYUP if wparam.0 == 0x20 && self.space => {
                self.up(false, Input::Space);
            }
            WM_KILLFOCUS if self.visible => {
                self.flush_click();
                self.click_start = None;
                self.doc.finish();
                self.space = false;
                self.gesture = None;
                let _ = ReleaseCapture();
                self.repaint();
            }
            WM_DISPLAYCHANGE if self.visible => self.close_overlay(),
            WM_CLOSE => {
                if self.visible {
                    self.close_overlay();
                } else {
                    let _ = DestroyWindow(self.hwnd);
                }
            }
            TRAY => match lparam.0 as u32 {
                WM_LBUTTONUP => {
                    if let Err(e) = self.activate() {
                        show_error(&e.to_string());
                    }
                }
                WM_RBUTTONUP | WM_CONTEXTMENU => self.tray_menu(),
                _ => {}
            },
            _ if msg == self.taskbar_created => {
                let _ = Shell_NotifyIconW(NIM_ADD, &self.tray);
            }
            _ => return None,
        }
        Some(LRESULT(0))
    }
}
unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if msg == WM_DESTROY {
        PostQuitMessage(0);
        return LRESULT(0);
    }
    // Native calls can synchronously re-enter the window procedure. Defer those messages to Win32.
    let result = APP.with(|app| {
        app.try_borrow_mut()
            .ok()
            .and_then(|mut app| app.as_mut().and_then(|a| a.message(msg, wparam, lparam)))
    });
    result.unwrap_or_else(|| DefWindowProcW(hwnd, msg, wparam, lparam))
}

pub fn show_error(message: &str) {
    unsafe {
        let text = HSTRING::from(message);
        MessageBoxW(None, &text, w!("Myosotis"), MB_OK | MB_ICONERROR);
    }
}

pub fn run() -> Result<()> {
    unsafe {
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;
        let instance = GetModuleHandleW(None)?;
        let class = w!("Myosotis.Native.Overlay");
        // One owner of the shortcut and tray. Launching again opens the resident app.
        let existing = FindWindowW(class, PCWSTR::null()).unwrap_or_default();
        if !existing.is_invalid() {
            PostMessageW(existing, WM_HOTKEY, WPARAM(HOTKEY as usize), LPARAM(0))?;
            CoUninitialize();
            return Ok(());
        }
        let wc = WNDCLASSW {
            lpfnWndProc: Some(wndproc),
            hInstance: instance.into(),
            lpszClassName: class,
            hCursor: LoadCursorW(None, IDC_CROSS)?,
            hIcon: LoadIconW(instance, PCWSTR(std::ptr::without_provenance(1)))?,
            ..Default::default()
        };
        if RegisterClassW(&wc) == 0 {
            return Err(Error::from_win32());
        }
        let hwnd = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
            class,
            w!("Myosotis — Capture & scribble"),
            WS_POPUP,
            0,
            0,
            1,
            1,
            None,
            None,
            instance,
            None,
        )?;
        if let Err(e) = RegisterHotKey(
            hwnd,
            HOTKEY,
            MOD_CONTROL | MOD_ALT | MOD_NOREPEAT,
            b'S' as u32,
        ) {
            let _ = DestroyWindow(hwnd);
            return Err(Error::new(e.code(), "Ctrl+Alt+S is already in use. Close the other app using that shortcut and start Myosotis again."));
        }
        let mut tray = NOTIFYICONDATAW {
            cbSize: size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: hwnd,
            uID: 1,
            uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
            uCallbackMessage: TRAY,
            hIcon: wc.hIcon,
            ..Default::default()
        };
        let tip: Vec<u16> = "Myosotis · Ctrl+Alt+S · Right-click for options"
            .encode_utf16()
            .collect();
        tray.szTip[..tip.len()].copy_from_slice(&tip);
        if !Shell_NotifyIconW(NIM_ADD, &tray).as_bool() {
            let _ = UnregisterHotKey(hwnd, HOTKEY);
            let _ = DestroyWindow(hwnd);
            return Err(Error::new(E_FAIL, "Could not add the Myosotis tray icon"));
        }
        let theme = settings::path()
            .and_then(|path| Theme::load(&path))
            .unwrap_or_else(|error| {
                eprintln!("Could not load theme settings; using Dark: {error}");
                Theme::Dark
            });
        APP.with(|a| {
            *a.borrow_mut() = Some(App {
                hwnd,
                previous: HWND::default(),
                doc: Document::default(),
                renderer: None,
                theme,
                pixels: vec![],
                origin: POINT::default(),
                full: Rect::default(),
                scale: 1.,
                visible: false,
                pointer: Point::default(),
                gesture: None,
                click_start: None,
                pending_click: None,
                space: false,
                hover: None,
                hint: String::new(),
                tray,
                taskbar_created: RegisterWindowMessageW(w!("TaskbarCreated")),
            })
        });
        if std::env::args().any(|arg| arg == "--capture") {
            PostMessageW(hwnd, WM_HOTKEY, WPARAM(HOTKEY as usize), LPARAM(0))?;
        }
        let mut msg = MSG::default();
        loop {
            let result = GetMessageW(&mut msg, None, 0, 0).0;
            if result <= 0 {
                break;
            }
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        APP.with(|a| {
            if let Some(app) = a.borrow_mut().take() {
                let _ = Shell_NotifyIconW(NIM_DELETE, &app.tray);
                let _ = UnregisterHotKey(app.hwnd, HOTKEY);
            }
        });
        CoUninitialize();
        Ok(())
    }
}
