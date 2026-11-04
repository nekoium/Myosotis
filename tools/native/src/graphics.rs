//! Shared native GDI+ backend. No visible windows or desktop access unless capture() is called.
use crate::{wide, Result};
use std::result::Result::Ok;
use std::{path::Path, ptr::null_mut};
use windows_sys::Win32::Graphics::{Gdi::*, GdiPlus::*};

const ARGB: i32 = 0x26200a;
fn check(status: Status) -> Result<()> {
    if status == 0 {
        Ok(())
    } else {
        Err(format!("GDI+ status {status}").into())
    }
}
pub struct Runtime(usize);
impl Runtime {
    pub fn new() -> Result<Self> {
        let input = GdiplusStartupInput {
            GdiplusVersion: 1,
            ..Default::default()
        };
        let mut token = 0;
        unsafe {
            check(GdiplusStartup(&mut token, &input, null_mut()))?;
        }
        Ok(Self(token))
    }
}
impl Drop for Runtime {
    fn drop(&mut self) {
        unsafe {
            GdiplusShutdown(self.0);
        }
    }
}

pub struct Image {
    raw: *mut GpBitmap,
    pub width: i32,
    pub height: i32,
}
impl Image {
    fn from_raw(raw: *mut GpBitmap) -> Result<Self> {
        let (mut w, mut h) = (0, 0);
        let mut image = Self {
            raw,
            width: 0,
            height: 0,
        };
        unsafe {
            check(GdipGetImageWidth(raw.cast(), &mut w))?;
            check(GdipGetImageHeight(raw.cast(), &mut h))?;
        }
        image.width = w as i32;
        image.height = h as i32;
        Ok(image)
    }
    pub fn new(width: i32, height: i32) -> Result<Self> {
        let mut raw = null_mut();
        unsafe {
            check(GdipCreateBitmapFromScan0(
                width,
                height,
                0,
                ARGB,
                null_mut(),
                &mut raw,
            ))?;
        }
        Self::from_raw(raw)
    }
    pub fn load(path: &Path) -> Result<Self> {
        let mut raw = null_mut();
        unsafe {
            check(GdipCreateBitmapFromFile(wide(path).as_ptr(), &mut raw))
                .map_err(|e| format!("{}: {e}", path.display()))?;
        }
        Self::from_raw(raw)
    }
    pub fn save(&self, path: &Path) -> Result<()> {
        let png = windows_sys::core::GUID::from_u128(0x557cf406_1a04_11d3_9a73_0000f81ef32e);
        unsafe {
            check(GdipSaveImageToFile(
                self.raw.cast(),
                wide(path).as_ptr(),
                &png,
                std::ptr::null(),
            ))
        }
    }
    pub fn pixels(&self) -> Result<Vec<u32>> {
        let mut data = BitmapData::default();
        let rect = Rect {
            X: 0,
            Y: 0,
            Width: self.width,
            Height: self.height,
        };
        unsafe {
            check(GdipBitmapLockBits(
                self.raw,
                &rect,
                ImageLockModeRead as u32,
                ARGB,
                &mut data,
            ))?;
            let mut pixels = Vec::with_capacity((self.width * self.height) as usize);
            for y in 0..self.height {
                let row = (data.Scan0 as *const u8).offset(y as isize * data.Stride as isize)
                    as *const u32;
                pixels.extend_from_slice(std::slice::from_raw_parts(row, self.width as usize));
            }
            check(GdipBitmapUnlockBits(self.raw, &mut data))?;
            Ok(pixels)
        }
    }
    /// Copies the provided GDI bitmap; caller retains ownership of the HBITMAP.
    ///
    /// # Safety
    /// `bitmap` must be a valid HBITMAP for the duration of this call.
    pub unsafe fn from_bitmap(bitmap: HBITMAP) -> Result<Self> {
        let mut raw = null_mut();
        unsafe {
            check(GdipCreateBitmapFromHBITMAP(bitmap, null_mut(), &mut raw))?;
        }
        Self::from_raw(raw)
    }
    pub fn capture(x: i32, y: i32, width: i32, height: i32) -> Result<Self> {
        unsafe {
            let screen = GetDC(null_mut());
            if screen.is_null() {
                return Err(std::io::Error::last_os_error().into());
            }
            let dc = CreateCompatibleDC(screen);
            let bitmap = CreateCompatibleBitmap(screen, width, height);
            let old = SelectObject(dc, bitmap);
            let copied = BitBlt(dc, 0, 0, width, height, screen, x, y, SRCCOPY);
            SelectObject(dc, old);
            let result = if copied == 0 {
                Err(std::io::Error::last_os_error().into())
            } else {
                Self::from_bitmap(bitmap)
            };
            DeleteObject(bitmap);
            DeleteDC(dc);
            ReleaseDC(null_mut(), screen);
            result
        }
    }
}
impl Drop for Image {
    fn drop(&mut self) {
        unsafe {
            GdipDisposeImage(self.raw.cast());
        }
    }
}

pub struct Canvas {
    graphics: *mut GpGraphics,
    image: Image,
}
impl Canvas {
    pub fn new(width: i32, height: i32) -> Result<Self> {
        let image = Image::new(width, height)?;
        unsafe {
            check(GdipBitmapSetResolution(image.raw, 144.0, 144.0))?;
        }
        let mut graphics = null_mut();
        unsafe {
            check(GdipGetImageGraphicsContext(image.raw.cast(), &mut graphics))?;
        }
        let canvas = Self { graphics, image };
        unsafe {
            check(GdipSetSmoothingMode(graphics, SmoothingModeAntiAlias))?;
            check(GdipSetInterpolationMode(
                graphics,
                InterpolationModeHighQualityBicubic,
            ))?;
            check(GdipSetPixelOffsetMode(graphics, PixelOffsetModeHighQuality))?;
            check(GdipSetTextRenderingHint(
                graphics,
                TextRenderingHintAntiAliasGridFit,
            ))?;
        }
        Ok(canvas)
    }
    pub fn clear(&self, color: u32) -> Result<()> {
        unsafe { check(GdipGraphicsClear(self.graphics, color)) }
    }
    pub fn fill(&self, color: u32, x: f32, y: f32, w: f32, h: f32) -> Result<()> {
        unsafe {
            let mut brush = null_mut();
            check(GdipCreateSolidFill(color, &mut brush))?;
            let result = check(GdipFillRectangle(self.graphics, brush.cast(), x, y, w, h));
            GdipDeleteBrush(brush.cast());
            result
        }
    }
    pub fn text(
        &self,
        value: &str,
        x: f32,
        y: f32,
        size: f32,
        color: u32,
        bold: bool,
    ) -> Result<()> {
        unsafe {
            let (mut family, mut font, mut brush, mut format) =
                (null_mut(), null_mut(), null_mut(), null_mut());
            check(GdipCreateFontFamilyFromName(
                wide("Segoe UI").as_ptr(),
                null_mut(),
                &mut family,
            ))?;
            let result = (|| {
                check(GdipCreateFont(
                    family,
                    size,
                    if bold {
                        FontStyleBold
                    } else {
                        FontStyleRegular
                    },
                    UnitPixel,
                    &mut font,
                ))?;
                check(GdipCreateSolidFill(color, &mut brush))?;
                check(GdipStringFormatGetGenericTypographic(&mut format))?;
                let text = wide(value);
                let layout = RectF {
                    X: x,
                    Y: y,
                    Width: 4000.0,
                    Height: 0.0,
                };
                let mut bounds = RectF::default();
                check(GdipMeasureString(
                    self.graphics,
                    text.as_ptr(),
                    (text.len() - 1) as i32,
                    font,
                    &layout,
                    format,
                    &mut bounds,
                    null_mut(),
                    null_mut(),
                ))?;
                if x < 0.0 || x + bounds.Width > 1780.0 {
                    return Err(format!("Text exceeds canvas: {value}").into());
                }
                let origin = RectF {
                    X: x,
                    Y: y,
                    Width: 0.0,
                    Height: 0.0,
                };
                check(GdipDrawString(
                    self.graphics,
                    text.as_ptr(),
                    (text.len() - 1) as i32,
                    font,
                    &origin,
                    format,
                    brush.cast(),
                ))
            })();
            if !format.is_null() {
                GdipDeleteStringFormat(format);
            }
            if !brush.is_null() {
                GdipDeleteBrush(brush.cast());
            }
            if !font.is_null() {
                GdipDeleteFont(font);
            }
            GdipDeleteFontFamily(family);
            result
        }
    }
    pub fn crop(&self, image: &Image, source: [f32; 4], target: [f32; 4]) -> Result<()> {
        let [sx, sy, sw, sh] = source;
        let [x, y, w, h] = target;
        if sx < 0.0 || sy < 0.0 || sx + sw > image.width as f32 || sy + sh > image.height as f32 {
            return Err("Invalid crop".into());
        }
        unsafe {
            check(GdipDrawImageRectRect(
                self.graphics,
                image.raw.cast(),
                x,
                y,
                w,
                h,
                sx,
                sy,
                sw,
                sh,
                UnitPixel,
                null_mut(),
                0,
                null_mut(),
            ))
        }
    }
    pub fn save(&self, path: &Path) -> Result<()> {
        self.image.save(path)
    }
}
impl Drop for Canvas {
    fn drop(&mut self) {
        unsafe {
            GdipDeleteGraphics(self.graphics);
        }
    }
}
