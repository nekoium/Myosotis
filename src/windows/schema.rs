use super::*;
use crate::input::{self, Action, Bindings, Shortcut};
use ::windows::Win32::UI::Controls::WM_MOUSELEAVE;

pub const APPLY: u32 = WM_APP + 2;
pub const WIDTH: f32 = 344.;
pub const HEIGHT: f32 = 294.;
const CLASS: PCWSTR = w!("Myosotis.Native.InputSchema");
const RESET: usize = 10;
const CLOSE: usize = 11;
thread_local! { static WINDOW: RefCell<Option<Schema>> = const { RefCell::new(None) }; }

// All rows share the same column calculation, including hit targets.
pub fn icon_rect(index: usize) -> Rect {
    let column = if index == 0 {
        0
    } else if index <= 6 {
        index - 1
    } else {
        index - 7
    };
    let x = 16. + (312. / 6.) * column as f32 + 9.;
    let y = if index == 0 {
        49.
    } else if index <= 6 {
        108.
    } else {
        206.
    };
    Rect {
        left: x,
        top: y,
        right: x + 34.,
        bottom: y + 32.,
    }
}
pub fn key_rect(index: usize, text_width: f32) -> Rect {
    if index == 0 {
        let width = (text_width + 14.).clamp(27., 235.);
        return Rect {
            left: 320. - width,
            top: 53.,
            right: 320.,
            bottom: 78.,
        };
    }
    let icon = icon_rect(index);
    let width = (text_width + 14.).clamp(27., 49.);
    let center = (icon.left + icon.right) / 2.;
    Rect {
        left: center - width / 2.,
        right: center + width / 2.,
        top: icon.bottom + 10.,
        bottom: icon.bottom + 35.,
    }
}
pub fn reset_rect() -> Rect {
    Rect {
        left: 270.,
        top: 11.,
        right: 296.,
        bottom: 39.,
    }
}
pub fn close_rect() -> Rect {
    Rect {
        left: 305.,
        top: 11.,
        right: 331.,
        bottom: 39.,
    }
}
pub fn label(bindings: &Bindings, index: usize) -> String {
    if index == 0 {
        bindings.capture.label()
    } else {
        bindings.label(Action::ALL[index - 1])
    }
}
fn point_in(lparam: LPARAM, scale: f32) -> Point {
    Point::new(
        (lparam.0 as u16 as i16) as f32 / scale,
        ((lparam.0 >> 16) as u16 as i16) as f32 / scale,
    )
}

pub struct Schema {
    hwnd: HWND,
    owner: HWND,
    pub bindings: Bindings,
    pub key_rects: [Rect; 10],
    pub theme: Theme,
    pub focus: Option<usize>,
    pub listening: Option<usize>,
    pub hover: Option<usize>,
    renderer: render::Renderer,
    scale: f32,
    window_size: (i32, i32),
    pending: Option<Bindings>,
    // Do not accept keys that were held before recording began.
    blocked: Vec<u16>,
}
impl Schema {
    fn refresh_keys(&mut self) {
        match unsafe { self.renderer.schema_keys(&self.bindings) } {
            Ok(keys) => self.key_rects = keys,
            Err(error) => eprintln!("Could not measure shortcuts: {error}"),
        }
    }
    fn hit(&self, point: Point) -> Option<usize> {
        if close_rect().contains(point) {
            return Some(CLOSE);
        }
        if reset_rect().contains(point) {
            return Some(RESET);
        }
        (0..10).find(|&index| self.key_rects[index].contains(point))
    }
    unsafe fn repaint(&self) {
        let _ = InvalidateRect(self.hwnd, None, false);
    }
    unsafe fn cancel(&mut self) {
        self.listening = None;
        self.blocked.clear();
    }
    unsafe fn hide(&mut self) {
        self.cancel();
        self.focus = None;
        self.hover = None;
        let _ = ShowWindow(self.hwnd, SW_HIDE);
    }
    unsafe fn request(&mut self, bindings: Bindings) {
        self.cancel();
        self.resize();
        if self.bindings == bindings {
            self.repaint();
            return;
        }
        self.pending = Some(bindings);
        let _ = PostMessageW(self.owner, APPLY, WPARAM(0), LPARAM(0));
    }
    unsafe fn activate(&mut self, index: usize) {
        self.cancel();
        self.focus = Some(index);
        match index {
            CLOSE => self.hide(),
            RESET if self.bindings != Bindings::default() => self.request(Bindings::default()),
            RESET => {}
            _ => {
                self.listening = Some(index);
                self.blocked = (1..=254)
                    .filter(|&key| GetAsyncKeyState(key as i32) < 0)
                    .collect();
            }
        }
        self.resize();
        self.repaint();
    }
    unsafe fn key(&mut self, key: u16, repeat: bool) {
        if repeat {
            return;
        }
        if let Some(index) = self.listening {
            if key == 0x09 {
                self.cancel();
            } else {
                if self.blocked.contains(&key) {
                    return;
                }
                let mut bindings = self.bindings.clone();
                let result = if index == 0 {
                    if input::is_modifier(key) {
                        return;
                    }
                    let mut modifiers = 0;
                    for (vk, flag) in [
                        (0x11, input::CTRL),
                        (0x12, input::ALT),
                        (0x10, input::SHIFT),
                        (0x5b, input::WIN),
                        (0x5c, input::WIN),
                    ] {
                        if GetKeyState(vk) < 0 {
                            modifiers |= flag;
                        }
                    }
                    bindings.capture = Shortcut { key, modifiers };
                    bindings.capture.validate()
                } else if (GetKeyState(0x11) < 0 && input::normalize_key(key) != 0x11)
                    || GetKeyState(0x12) < 0
                    || GetKeyState(0x10) < 0
                    || GetKeyState(0x5b) < 0
                    || GetKeyState(0x5c) < 0
                {
                    Err("Use one key without modifiers.".into())
                } else {
                    bindings.set(Action::ALL[index - 1], key)
                };
                match result {
                    Ok(()) => self.request(bindings),
                    Err(_) => return,
                }
                self.resize();
                self.repaint();
                return;
            }
        }
        match key {
            0x09 => {
                let backwards = GetKeyState(0x10) < 0;
                let mut index = self.focus.unwrap_or(if backwards { 0 } else { 11 });
                loop {
                    index = if backwards {
                        (index + 11) % 12
                    } else {
                        (index + 1) % 12
                    };
                    if index != RESET || self.bindings != Bindings::default() {
                        break;
                    }
                }
                self.cancel();
                self.resize();
                self.focus = Some(index);
            }
            0x0d | 0x20 => {
                if let Some(index) = self.focus {
                    self.activate(index)
                }
            }
            0x1b => self.hide(),
            _ => {}
        }
        self.repaint();
    }
    unsafe fn resize(&mut self) {
        let height = HEIGHT;
        let size = (
            (WIDTH * self.scale).round() as i32,
            (height * self.scale).round() as i32,
        );
        if self.window_size == size {
            return;
        }
        self.window_size = size;
        let _ = SetWindowPos(
            self.hwnd,
            None,
            0,
            0,
            (WIDTH * self.scale).round() as i32,
            (height * self.scale).round() as i32,
            SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
        );
        let _ = self.renderer.target.Resize(
            &::windows::Win32::Graphics::Direct2D::Common::D2D_SIZE_U {
                width: (WIDTH * self.scale).round() as u32,
                height: (height * self.scale).round() as u32,
            },
        );
        let region = CreateRoundRectRgn(
            0,
            0,
            (WIDTH * self.scale).round() as i32 + 1,
            (height * self.scale).round() as i32 + 1,
            (20. * self.scale) as i32,
            (20. * self.scale) as i32,
        );
        if SetWindowRgn(self.hwnd, region, true) == 0 {
            let _ = DeleteObject(region);
        }
    }
    unsafe fn message(&mut self, msg: u32, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        match msg {
            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                BeginPaint(self.hwnd, &mut ps);
                let result = self.renderer.paint_schema(self);
                let _ = EndPaint(self.hwnd, &ps);
                if let Err(error) = result {
                    eprintln!("Input schema rendering failed: {error}");
                }
            }
            WM_ERASEBKGND => return Some(LRESULT(1)),
            WM_NCHITTEST => {
                let mut p = POINT {
                    x: lparam.0 as u16 as i16 as i32,
                    y: (lparam.0 >> 16) as u16 as i16 as i32,
                };
                let _ = ScreenToClient(self.hwnd, &mut p);
                let point = Point::new(p.x as f32 / self.scale, p.y as f32 / self.scale);
                if point.y < 42. && self.hit(point).is_none() {
                    return Some(LRESULT(HTCAPTION as isize));
                }
                return Some(LRESULT(HTCLIENT as isize));
            }
            WM_SETCURSOR => {
                let clickable = self
                    .hover
                    .is_some_and(|index| index != RESET || self.bindings != Bindings::default());
                if let Ok(cursor) = LoadCursorW(None, if clickable { IDC_HAND } else { IDC_ARROW })
                {
                    SetCursor(cursor);
                }
                return Some(LRESULT(1));
            }
            WM_MOUSEMOVE => {
                let point = point_in(lparam, self.scale);
                let next = self.hit(point);
                if self.hover != next {
                    self.hover = next;
                    self.repaint();
                }
                let mut track = TRACKMOUSEEVENT {
                    cbSize: size_of::<TRACKMOUSEEVENT>() as u32,
                    dwFlags: TME_LEAVE,
                    hwndTrack: self.hwnd,
                    ..Default::default()
                };
                let _ = TrackMouseEvent(&mut track);
            }
            WM_MOUSELEAVE => {
                self.hover = None;
                self.repaint();
            }
            WM_LBUTTONDOWN => {
                let _ = SetFocus(self.hwnd);
                if let Some(index) = self.hit(point_in(lparam, self.scale)) {
                    self.activate(index);
                } else {
                    self.cancel();
                    self.focus = None;
                    self.repaint();
                }
            }
            WM_SYSKEYDOWN
                if self.listening.is_none() && wparam.0 == 0x73 && GetKeyState(0x12) < 0 =>
            {
                self.hide()
            }
            WM_KEYDOWN | WM_SYSKEYDOWN => self.key(wparam.0 as u16, lparam.0 & (1 << 30) != 0),
            WM_KEYUP | WM_SYSKEYUP => self.blocked.retain(|&key| key != wparam.0 as u16),
            WM_KILLFOCUS => self.hide(),
            WM_CLOSE => self.hide(),
            WM_DPICHANGED => {
                self.scale = (wparam.0 & 0xffff) as f32 / 96.;
                let r = &*(lparam.0 as *const RECT);
                let _ = SetWindowPos(
                    self.hwnd,
                    None,
                    r.left,
                    r.top,
                    (WIDTH * self.scale) as i32,
                    (HEIGHT * self.scale) as i32,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                );
                if let Ok(renderer) = new_renderer(self.hwnd, self.scale, self.theme) {
                    self.renderer = renderer;
                }
                self.refresh_keys();
                self.resize();
                self.repaint();
            }
            _ => return None,
        }
        Some(LRESULT(0))
    }
}
unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let result = WINDOW.with(|window| {
        window.try_borrow_mut().ok().and_then(|mut window| {
            window
                .as_mut()
                .and_then(|window| window.message(msg, wparam, lparam))
        })
    });
    result.unwrap_or_else(|| DefWindowProcW(hwnd, msg, wparam, lparam))
}
unsafe fn new_renderer(hwnd: HWND, scale: f32, theme: Theme) -> Result<render::Renderer> {
    let width = (WIDTH * scale).round() as u32;
    let height = (HEIGHT * scale).round() as u32;
    render::Renderer::new(
        hwnd,
        width,
        height,
        &vec![0; (width * height * 4) as usize],
        scale,
        theme,
    )
}
pub unsafe fn open(owner: HWND, theme: Theme, bindings: &Bindings) -> Result<()> {
    if WINDOW.with(|w| {
        let mut window = w.borrow_mut();
        if let Some(window) = window.as_mut() {
            window.bindings = bindings.clone();
            window.theme = theme;
            window.renderer.theme = theme;
            window.refresh_keys();
            window.resize();
            let _ = ShowWindow(window.hwnd, SW_SHOWNORMAL);
            let _ = SetForegroundWindow(window.hwnd);
            let _ = SetFocus(window.hwnd);
            window.repaint();
            true
        } else {
            false
        }
    }) {
        return Ok(());
    }
    let instance = GetModuleHandleW(None)?;
    let _ = RegisterClassW(&WNDCLASSW {
        lpfnWndProc: Some(wndproc),
        hInstance: instance.into(),
        lpszClassName: CLASS,
        hCursor: LoadCursorW(None, IDC_ARROW)?,
        hIcon: LoadIconW(instance, PCWSTR(std::ptr::without_provenance(1)))?,
        ..Default::default()
    });
    let mut pointer = POINT::default();
    let _ = GetCursorPos(&mut pointer);
    let monitor = MonitorFromPoint(pointer, MONITOR_DEFAULTTONEAREST);
    let mut info = MONITORINFO {
        cbSize: size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    if !GetMonitorInfoW(monitor, &mut info).as_bool() {
        return Err(Error::from_win32());
    }
    let (mut dpi, mut y) = (96, 96);
    let _ = GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut dpi, &mut y);
    let scale = dpi as f32 / 96.;
    let width = (WIDTH * scale).round() as i32;
    let height = (HEIGHT * scale).round() as i32;
    let hwnd = CreateWindowExW(
        WS_EX_TOOLWINDOW,
        CLASS,
        w!("Myosotis — Input schema"),
        WS_POPUP | WS_SYSMENU,
        info.rcWork.left + (info.rcWork.right - info.rcWork.left - width) / 2,
        info.rcWork.top + (info.rcWork.bottom - info.rcWork.top - height) / 2,
        width,
        height,
        None,
        None,
        instance,
        None,
    )?;
    let setup = (|| -> Result<Schema> {
        disable_ime(hwnd)?;
        let renderer = new_renderer(hwnd, scale, theme)?;
        let key_rects = renderer.schema_keys(bindings)?;
        Ok(Schema {
            hwnd,
            owner,
            bindings: bindings.clone(),
            key_rects,
            theme,
            focus: None,
            listening: None,
            hover: None,
            renderer,
            scale,
            window_size: (0, 0),
            pending: None,
            blocked: Vec::new(),
        })
    })();
    let window = match setup {
        Ok(w) => w,
        Err(error) => {
            let _ = DestroyWindow(hwnd);
            return Err(error);
        }
    };
    WINDOW.with(|w| *w.borrow_mut() = Some(window));
    WINDOW.with(|w| {
        let mut window = w.borrow_mut();
        let window = window.as_mut().unwrap();
        window.resize();
        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = SetForegroundWindow(hwnd);
        let _ = SetFocus(hwnd);
        window.repaint();
    });
    Ok(())
}
pub fn recording() -> bool {
    WINDOW.with(|w| w.borrow().as_ref().is_some_and(|w| w.listening.is_some()))
}
pub unsafe fn hide() {
    WINDOW.with(|w| {
        if let Some(w) = w.borrow_mut().as_mut() {
            w.hide();
        }
    });
}
pub unsafe fn update(theme: Theme, bindings: &Bindings) {
    WINDOW.with(|w| {
        if let Some(w) = w.borrow_mut().as_mut() {
            w.theme = theme;
            w.renderer.theme = theme;
            w.bindings = bindings.clone();
            w.refresh_keys();
            w.resize();
            w.repaint();
        }
    });
}
pub fn take_request() -> Option<Bindings> {
    WINDOW.with(|w| w.borrow_mut().as_mut().and_then(|w| w.pending.take()))
}
pub unsafe fn close() {
    WINDOW.with(|w| {
        if let Some(w) = w.borrow_mut().take() {
            let _ = DestroyWindow(w.hwnd);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dismissal_clears_recording_and_focus_and_reset_requests_only_defaults() -> Result<()> {
        unsafe {
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("STATIC"),
                w!("Myosotis schema state test"),
                WS_POPUP,
                0,
                0,
                WIDTH as i32,
                HEIGHT as i32,
                None,
                None,
                None,
                None,
            )?;
            let result = (|| -> Result<()> {
                let mut bindings = Bindings::default();
                bindings.set(Action::Pen, b'Q' as u16).unwrap();
                let renderer = new_renderer(hwnd, 1., Theme::Light)?;
                let key_rects = renderer.schema_keys(&bindings)?;
                let mut window = Schema {
                    hwnd,
                    owner: hwnd,
                    renderer,
                    key_rects,
                    bindings: bindings.clone(),
                    theme: Theme::Light,
                    focus: Some(1),
                    listening: Some(1),
                    hover: Some(1),
                    scale: 1.,
                    window_size: (0, 0),
                    pending: None,
                    blocked: vec![b'W' as u16],
                };
                window.key(b'W' as u16, false);
                window.key(b'E' as u16, true);
                assert_eq!(window.listening, Some(1));
                assert!(window.pending.is_none());
                let original_size = window.window_size;
                window.key(0x20, false);
                assert_eq!(window.listening, Some(1));
                assert_eq!(window.bindings, bindings);
                assert!(window.pending.is_none());
                assert_eq!(window.window_size, original_size);
                assert!(IsWindow(hwnd).as_bool());
                window.key(0x1b, false);
                assert_eq!(
                    window.listening,
                    Some(1),
                    "duplicate Escape keeps recording"
                );
                assert_eq!(window.focus, Some(1));
                assert_eq!(window.hover, Some(1));
                assert!(window.pending.is_none());

                window.bindings.set(Action::Cancel, b'F' as u16).unwrap();
                window.listening = Some(Action::Cancel as usize + 1);
                window.key(0x1b, false);
                assert_eq!(window.listening, None);
                assert!(window.blocked.is_empty());
                assert_eq!(window.focus, Some(1), "recording Escape must not hide");
                assert_eq!(window.hover, Some(1));
                assert_eq!(window.pending.take(), Some(bindings.clone()));
                window.bindings = bindings.clone();
                window.focus = Some(0);
                window.hover = Some(0);
                window.blocked.push(b'W' as u16);
                window.listening = Some(0);
                window.message(WM_KILLFOCUS, WPARAM(0), LPARAM(0));
                assert_eq!(window.listening, None);
                assert_eq!(window.focus, None);
                assert_eq!(window.hover, None);
                assert!(window.blocked.is_empty());
                assert!(window.pending.is_none());
                window.focus = Some(CLOSE);
                window.hover = Some(CLOSE);
                window.key(0x1b, false);
                assert_eq!(window.focus, None);
                assert_eq!(window.hover, None);
                assert!(
                    IsWindow(hwnd).as_bool(),
                    "dismissal keeps the reusable window"
                );
                window.activate(RESET);
                assert_eq!(window.pending, Some(Bindings::default()));
                assert_eq!(
                    window.bindings, bindings,
                    "do not apply before registration and persistence succeed"
                );
                assert_eq!(window.theme, Theme::Light);
                window.pending.take();
                window.listening = Some(1);
                window.request(bindings.clone());
                assert!(window.pending.is_none());
                assert_eq!(window.listening, None);
                Ok(())
            })();
            let _ = DestroyWindow(hwnd);
            result
        }
    }

    #[test]
    fn columns_align_and_keys_have_nonoverlapping_hit_targets() {
        for index in 7..10 {
            assert_eq!(icon_rect(index).left, icon_rect(index - 6).left);
        }
        for index in 1..10 {
            let icon = icon_rect(index);
            let key = key_rect(index, 13.);
            assert_eq!(icon.left + icon.right, key.left + key.right);
            assert!(key.bottom < HEIGHT);
        }
        assert_eq!(key_rect(0, 63.).right, 320.);
        assert_eq!(HEIGHT - key_rect(7, 13.).bottom, 21.);
        assert!(reset_rect().right < close_rect().left);
    }
}
