//! GDI acquisition and CF_DIB transfer; bitmap/DC ownership stays in this module.
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
    },
};
use std::{ffi::c_void, mem::size_of};

pub struct Surface {
    pub dc: HDC,
    bitmap: HBITMAP,
    old: HGDIOBJ,
    bits: *mut c_void,
    pub width: i32,
    pub height: i32,
}
impl Surface {
    pub unsafe fn new(width: i32, height: i32) -> Result<Self> {
        if width <= 0 || height <= 0 || (width as u64 * height as u64) > 200_000_000 {
            return Err(Error::new(
                E_INVALIDARG,
                "Invalid or excessively large image dimensions",
            ));
        }
        let dc = CreateCompatibleDC(None);
        if dc.is_invalid() {
            return Err(Error::from_win32());
        }
        let info = BITMAPINFO {
            bmiHeader: header(width, height),
            ..Default::default()
        };
        let mut bits = std::ptr::null_mut();
        let bitmap = match CreateDIBSection(dc, &info, DIB_RGB_COLORS, &mut bits, None, 0) {
            Ok(b) => b,
            Err(e) => {
                let _ = DeleteDC(dc);
                return Err(e);
            }
        };
        let old = SelectObject(dc, bitmap);
        Ok(Self {
            dc,
            bitmap,
            old,
            bits,
            width,
            height,
        })
    }
    pub unsafe fn pixels(&self) -> &[u8] {
        std::slice::from_raw_parts(
            self.bits.cast(),
            self.width as usize * self.height as usize * 4,
        )
    }
    pub unsafe fn capture(x: i32, y: i32, width: i32, height: i32) -> Result<Vec<u8>> {
        let surface = Self::new(width, height)?;
        let screen = GetDC(None);
        if screen.is_invalid() {
            return Err(Error::from_win32());
        }
        let result = BitBlt(
            surface.dc,
            0,
            0,
            width,
            height,
            screen,
            x,
            y,
            SRCCOPY | CAPTUREBLT,
        );
        ReleaseDC(None, screen);
        result?;
        let mut pixels = surface.pixels().to_vec();
        for pixel in pixels.as_chunks_mut::<4>().0 {
            pixel[3] = 255;
        }
        Ok(pixels)
    }
    pub unsafe fn to_clipboard(&self, owner: HWND) -> Result<()> {
        // Encode before opening the clipboard, so an encoding error preserves existing data.
        let png = self.png()?;
        let png_format = RegisterClipboardFormatW(w!("PNG"));
        if png_format == 0 {
            return Err(Error::from_win32());
        }
        let stride = self.width as usize * 4;
        let mut dib = Vec::with_capacity(size_of::<BITMAPINFOHEADER>() + self.pixels().len());
        let mut info = header(self.width, self.height);
        // Bottom-up DIBs also work with older clipboard readers and Windows Forms.
        info.biHeight = self.height;
        dib.extend_from_slice(std::slice::from_raw_parts(
            (&info as *const BITMAPINFOHEADER).cast::<u8>(),
            size_of::<BITMAPINFOHEADER>(),
        ));
        for row in self.pixels().chunks_exact(stride).rev() {
            for pixel in row.as_chunks::<4>().0 {
                dib.extend_from_slice(&[pixel[0], pixel[1], pixel[2], 255]);
            }
        }
        let dib = ClipboardMemory::new(&dib)?;
        let png = ClipboardMemory::new(&png)?;
        OpenClipboard(owner)?;
        let result = (|| {
            EmptyClipboard()?;
            dib.transfer(8)?;
            // CF_DIB is already available if an optional PNG transfer fails.
            if let Err(error) = png.transfer(png_format) {
                eprintln!("PNG clipboard format: {error}");
            }
            Ok(())
        })();
        let _ = CloseClipboard();
        result
    }
    unsafe fn png(&self) -> Result<Vec<u8>> {
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
        frame.SetSize(self.width as u32, self.height as u32)?;
        let mut format = GUID_WICPixelFormat32bppBGRA;
        frame.SetPixelFormat(&mut format)?;
        if format != GUID_WICPixelFormat32bppBGRA {
            return Err(Error::new(
                E_FAIL,
                "PNG encoder does not support BGRA pixels",
            ));
        }
        let mut pixels = self.pixels().to_vec();
        for pixel in pixels.as_chunks_mut::<4>().0 {
            pixel[3] = 255;
        }
        frame.WritePixels(self.height as u32, self.width as u32 * 4, &pixels)?;
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

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            SelectObject(self.dc, self.old);
            let _ = DeleteObject(self.bitmap);
            let _ = DeleteDC(self.dc);
        }
    }
}
fn header(width: i32, height: i32) -> BITMAPINFOHEADER {
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
