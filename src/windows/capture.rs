//! GDI acquisition; bitmap/DC ownership stays in this module.
use ::windows::{
    core::*,
    Win32::{Foundation::*, Graphics::Gdi::*},
};
use std::ffi::c_void;

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
            bmiHeader: super::clipboard::header(width, height),
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
        // Complete queued GDI writes before reading the DIB memory directly.
        GdiFlush().ok()?;
        let mut pixels = surface.pixels().to_vec();
        for pixel in pixels.as_chunks_mut::<4>().0 {
            pixel[3] = 255;
        }
        Ok(pixels)
    }
    pub unsafe fn to_clipboard(&self, owner: HWND) -> Result<super::clipboard::Publication> {
        GdiFlush().ok()?;
        super::clipboard::publish(
            owner,
            super::clipboard::Image::new(self.width, self.height, self.pixels())?,
        )
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
