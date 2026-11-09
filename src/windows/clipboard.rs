//! Bitmap-first clipboard ownership and delayed, background-encoded PNG.
use ::windows::{
    core::*,
    Win32::{
        Foundation::*,
        Graphics::{Gdi::*, Imaging::*},
        System::{
            Com::{StructuredStorage::*, *},
            DataExchange::*,
            Memory::*,
        },
        UI::WindowsAndMessaging::*,
    },
};
use std::{cell::RefCell, mem::size_of, sync::Arc, thread::JoinHandle};

const PNG_TIMER: usize = 0x504e47;
thread_local! { static PENDING: RefCell<Option<Pending>> = const { RefCell::new(None) }; }

pub(super) struct Image {
    width: i32,
    height: i32,
    pixels: Vec<u8>,
}
impl Image {
    pub fn new(width: i32, height: i32, pixels: &[u8]) -> Result<Self> {
        if width <= 0
            || height <= 0
            || width as u64 * height as u64 > 200_000_000
            || pixels.len() != width as usize * height as usize * 4
        {
            return Err(Error::new(
                E_INVALIDARG,
                "Invalid clipboard image dimensions",
            ));
        }
        let mut pixels = pixels.to_vec();
        for pixel in pixels.as_chunks_mut::<4>().0 {
            pixel[3] = 255;
        }
        Ok(Self {
            width,
            height,
            pixels,
        })
    }
    fn dib(&self) -> Vec<u8> {
        let mut info = header(self.width, self.height);
        // Bottom-up DIBs also work with older clipboard readers and Windows Forms.
        info.biHeight = self.height;
        let mut bytes = Vec::with_capacity(size_of::<BITMAPINFOHEADER>() + self.pixels.len());
        unsafe {
            bytes.extend_from_slice(std::slice::from_raw_parts(
                (&info as *const BITMAPINFOHEADER).cast(),
                size_of::<BITMAPINFOHEADER>(),
            ));
        }
        for row in self.pixels.chunks_exact(self.width as usize * 4).rev() {
            bytes.extend_from_slice(row);
        }
        bytes
    }
}

struct Pending {
    owner: HWND,
    format: u32,
    image: Arc<Image>,
    job: Option<JoinHandle<Result<Vec<u8>>>>,
}
impl Pending {
    fn start(&mut self) {
        if self.job.is_some() {
            return;
        }
        let image = self.image.clone();
        match std::thread::Builder::new()
            .name("clipboard-png".into())
            .spawn(move || encode(&image))
        {
            Ok(job) => self.job = Some(job),
            Err(error) => {
                eprintln!("Could not start PNG encoder; encoding on request instead: {error}")
            }
        }
    }
    fn bytes(&mut self) -> Result<Vec<u8>> {
        match self.job.take() {
            Some(job) => job
                .join()
                .unwrap_or_else(|_| Err(Error::new(E_FAIL, "PNG encoder thread failed"))),
            None => encode(&self.image),
        }
    }
    unsafe fn current(&self) -> bool {
        // WM_DESTROYCLIPBOARD clears the previous job, even when the same HWND copies again.
        // Only the current UI-thread job can publish; workers never touch the clipboard.
        GetClipboardOwner().ok() == Some(self.owner)
    }
}

// Keep the clipboard locked until the overlay is hidden; clipboard listeners can
// synchronously request PNG as soon as CloseClipboard announces the new item.
pub(super) struct Publication;
impl Drop for Publication {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseClipboard();
        }
    }
}

pub(super) unsafe fn publish(owner: HWND, image: Image) -> Result<Publication> {
    // Prepare before opening the clipboard, so preparation errors preserve existing data.
    let dib = ClipboardMemory::new(&image.dib())?;
    let format = RegisterClipboardFormatW(w!("PNG"));
    if format == 0 {
        return Err(Error::from_win32());
    }
    OpenClipboard(owner)?;
    let result = (|| {
        EmptyClipboard()?;
        dib.transfer(8)?;
        // A NULL handle advertises delayed rendering; success also returns NULL.
        SetLastError(WIN32_ERROR(0));
        let _ = SetClipboardData(format, HANDLE::default());
        if GetLastError() != WIN32_ERROR(0) {
            eprintln!(
                "Could not advertise delayed PNG; bitmap remains available: {}",
                Error::from_win32()
            );
            return Ok(());
        }
        PENDING.with(|slot| {
            *slot.borrow_mut() = Some(Pending {
                owner,
                format,
                image: Arc::new(image),
                job: None,
            })
        });
        Ok(())
    })();
    match result {
        Ok(()) => Ok(Publication),
        Err(error) => {
            let _ = CloseClipboard();
            Err(error)
        }
    }
}

pub(super) fn start_encoding() {
    PENDING.with(|slot| {
        if let Some(pending) = slot.borrow_mut().as_mut() {
            pending.start();
            // Polling avoids worker notifications targeting a destroyed/reused HWND.
            if pending.job.is_some() {
                unsafe {
                    SetTimer(pending.owner, PNG_TIMER, 10, None);
                }
            }
        }
    });
}

unsafe fn render(hwnd: HWND, requested: Option<u32>, open: bool) {
    let Some(mut pending_value) = PENDING.with(|slot| slot.borrow_mut().take()) else {
        return;
    };
    if pending_value.owner != hwnd || !pending_value.current() {
        let _ = KillTimer(hwnd, PNG_TIMER);
        return;
    }
    if requested.is_some_and(|format| format != pending_value.format) {
        PENDING.with(|slot| *slot.borrow_mut() = Some(pending_value));
        return;
    }
    // WM_RENDERFORMAT arrives with the requesting process holding the clipboard open.
    // Never OpenClipboard here; a direct paste may wait for the in-flight worker.
    if open && OpenClipboard(hwnd).is_err() {
        PENDING.with(|slot| *slot.borrow_mut() = Some(pending_value));
        return;
    }
    if pending_value.current() {
        let result = pending_value
            .bytes()
            .and_then(|bytes| ClipboardMemory::new(&bytes))
            .and_then(|memory| memory.transfer(pending_value.format));
        if let Err(error) = result {
            eprintln!("PNG clipboard format: {error}; bitmap remains available");
        }
    }
    if open {
        let _ = CloseClipboard();
    }
    let _ = KillTimer(hwnd, PNG_TIMER);
}

pub(super) unsafe fn message(hwnd: HWND, msg: u32, wparam: WPARAM) -> Option<LRESULT> {
    match msg {
        WM_RENDERFORMAT => render(hwnd, Some(wparam.0 as u32), false),
        WM_RENDERALLFORMATS => render(hwnd, None, true),
        WM_DESTROYCLIPBOARD => {
            PENDING.with(|slot| {
                slot.borrow_mut().take();
            });
            let _ = KillTimer(hwnd, PNG_TIMER);
        }
        WM_TIMER if wparam.0 == PNG_TIMER => {
            let ready = PENDING.with(|slot| {
                slot.borrow().as_ref().is_some_and(|pending| {
                    pending.job.as_ref().is_some_and(JoinHandle::is_finished)
                })
            });
            if ready {
                render(hwnd, None, true);
            }
        }
        _ => return None,
    }
    Some(LRESULT(0))
}
pub(super) unsafe fn shutdown(hwnd: HWND) {
    render(hwnd, None, true);
}

fn encode(image: &Image) -> Result<Vec<u8>> {
    unsafe {
        let initialized = CoInitializeEx(None, COINIT_MULTITHREADED);
        if initialized == RPC_E_CHANGED_MODE {
            // A thread-creation failure can fall back to the already initialized UI apartment.
            return encode_initialized(image);
        }
        initialized.ok()?;
        let result = encode_initialized(image);
        CoUninitialize();
        result
    }
}
unsafe fn encode_initialized(image: &Image) -> Result<Vec<u8>> {
    let factory: IWICImagingFactory =
        CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER)?;
    let stream = CreateStreamOnHGlobal(None, true)?;
    let encoder = factory.CreateEncoder(&GUID_ContainerFormatPng, std::ptr::null())?;
    encoder.Initialize(&stream, WICBitmapEncoderNoCache)?;
    let mut frame = None;
    let mut options = None;
    encoder.CreateNewFrame(&mut frame, &mut options)?;
    let frame = frame.ok_or_else(|| Error::new(E_FAIL, "PNG encoder returned no frame"))?;
    frame.Initialize(options.as_ref())?;
    frame.SetSize(image.width as u32, image.height as u32)?;
    let mut format = GUID_WICPixelFormat32bppBGRA;
    frame.SetPixelFormat(&mut format)?;
    if format != GUID_WICPixelFormat32bppBGRA {
        return Err(Error::new(
            E_FAIL,
            "PNG encoder does not support BGRA pixels",
        ));
    }
    frame.WritePixels(image.height as u32, image.width as u32 * 4, &image.pixels)?;
    frame.Commit()?;
    encoder.Commit()?;
    let memory = GetHGlobalFromStream(&stream)?;
    let mut stat = STATSTG::default();
    stream.Stat(&mut stat, STATFLAG_NONAME)?;
    let ptr = GlobalLock(memory);
    if ptr.is_null() {
        return Err(Error::from_win32());
    }
    let bytes = std::slice::from_raw_parts(ptr.cast::<u8>(), stat.cbSize as usize).to_vec();
    let _ = GlobalUnlock(memory);
    Ok(bytes)
}

struct ClipboardMemory(HGLOBAL);
impl ClipboardMemory {
    unsafe fn new(bytes: &[u8]) -> Result<Self> {
        let memory = Self(GlobalAlloc(GMEM_MOVEABLE, bytes.len())?);
        let ptr = GlobalLock(memory.0);
        if ptr.is_null() {
            return Err(Error::from_win32());
        }
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr.cast::<u8>(), bytes.len());
        let _ = GlobalUnlock(memory.0);
        Ok(memory)
    }
    unsafe fn transfer(self, format: u32) -> Result<()> {
        SetClipboardData(format, HANDLE(self.0 .0))?;
        std::mem::forget(self);
        Ok(())
    }
}
impl Drop for ClipboardMemory {
    fn drop(&mut self) {
        unsafe {
            let _ = GlobalFree(self.0);
        }
    }
}

pub(super) fn header(width: i32, height: i32) -> BITMAPINFOHEADER {
    BITMAPINFOHEADER {
        biSize: size_of::<BITMAPINFOHEADER>() as u32,
        biWidth: width,
        biHeight: -height,
        biPlanes: 1,
        biBitCount: 32,
        biCompression: BI_RGB.0,
        ..Default::default()
    }
}

#[cfg(test)]
#[path = "clipboard_native_tests.rs"]
mod native_tests;
#[cfg(test)]
#[path = "clipboard_tests.rs"]
mod tests;
