use crate::document::{Annotation, Document, Point, Rect, Tool};
use ::windows::{
    core::*,
    Foundation::Numerics::Matrix3x2,
    Win32::{
        Foundation::*,
        Graphics::{
            Direct2D::{Common::*, *},
            DirectWrite::*,
            Dxgi::Common::*,
        },
    },
};

fn color(r: f32, g: f32, b: f32, a: f32) -> D2D1_COLOR_F {
    D2D1_COLOR_F { r, g, b, a }
}
fn rect(r: Rect) -> D2D_RECT_F {
    D2D_RECT_F {
        left: r.left,
        top: r.top,
        right: r.right,
        bottom: r.bottom,
    }
}
fn point(p: Point) -> D2D_POINT_2F {
    D2D_POINT_2F { x: p.x, y: p.y }
}
fn properties() -> D2D1_RENDER_TARGET_PROPERTIES {
    D2D1_RENDER_TARGET_PROPERTIES {
        pixelFormat: D2D1_PIXEL_FORMAT {
            format: DXGI_FORMAT_B8G8R8A8_UNORM,
            alphaMode: D2D1_ALPHA_MODE_IGNORE,
        },
        dpiX: 96.,
        dpiY: 96.,
        ..Default::default()
    }
}

pub struct Renderer {
    factory: ID2D1Factory,
    pub target: ID2D1HwndRenderTarget,
    bitmap: ID2D1Bitmap,
    small: IDWriteTextFormat,
    number: IDWriteTextFormat,
    scale: f32,
}
impl Renderer {
    pub unsafe fn new(
        hwnd: HWND,
        width: u32,
        height: u32,
        pixels: &[u8],
        scale: f32,
    ) -> Result<Self> {
        let factory: ID2D1Factory = D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)?;
        let target = factory.CreateHwndRenderTarget(
            &properties(),
            &D2D1_HWND_RENDER_TARGET_PROPERTIES {
                hwnd,
                pixelSize: D2D_SIZE_U { width, height },
                presentOptions: D2D1_PRESENT_OPTIONS_NONE,
            },
        )?;
        let bitmap = target.CreateBitmap(
            D2D_SIZE_U { width, height },
            Some(pixels.as_ptr().cast()),
            width * 4,
            &D2D1_BITMAP_PROPERTIES {
                pixelFormat: properties().pixelFormat,
                dpiX: 96.,
                dpiY: 96.,
            },
        )?;
        let write: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)?;
        let small = write.CreateTextFormat(
            w!("Segoe UI"),
            None,
            DWRITE_FONT_WEIGHT_NORMAL,
            DWRITE_FONT_STYLE_NORMAL,
            DWRITE_FONT_STRETCH_NORMAL,
            13. * scale,
            w!("en-us"),
        )?;
        small.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER)?;
        let number = write.CreateTextFormat(
            w!("Segoe UI"),
            None,
            DWRITE_FONT_WEIGHT_BOLD,
            DWRITE_FONT_STYLE_NORMAL,
            DWRITE_FONT_STRETCH_NORMAL,
            15. * scale,
            w!("en-us"),
        )?;
        number.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER)?;
        number.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER)?;
        Ok(Self {
            factory,
            target,
            bitmap,
            small,
            number,
            scale,
        })
    }
    unsafe fn text(
        &self,
        target: &ID2D1RenderTarget,
        text: &str,
        bounds: Rect,
        brush: &ID2D1SolidColorBrush,
        number: bool,
    ) {
        let text: Vec<u16> = text.encode_utf16().collect();
        target.DrawText(
            &text,
            if number { &self.number } else { &self.small },
            &rect(bounds),
            brush,
            D2D1_DRAW_TEXT_OPTIONS_CLIP,
            DWRITE_MEASURING_MODE_NATURAL,
        );
    }
    unsafe fn image(
        &self,
        target: &ID2D1RenderTarget,
        bitmap: &ID2D1Bitmap,
        doc: &Document,
        full: Rect,
    ) -> Result<()> {
        let white = target.CreateSolidColorBrush(&color(1., 1., 1., 1.), None)?;
        target.DrawBitmap(
            bitmap,
            Some(&rect(full)),
            1.,
            D2D1_BITMAP_INTERPOLATION_MODE_NEAREST_NEIGHBOR,
            None,
        );
        if let Some(selection) = doc.selection {
            target.PushAxisAlignedClip(&rect(selection), D2D1_ANTIALIAS_MODE_ALIASED);
            if doc.blank {
                target.FillRectangle(&rect(selection), &white);
            }
            let ink = target.CreateSolidColorBrush(&color(0.96, 0.22, 0.30, 1.), None)?;
            for a in doc.annotations.iter().chain(doc.active.iter()) {
                self.annotation(target, a, &ink, &white);
            }
            target.PopAxisAlignedClip();
        }
        Ok(())
    }
    unsafe fn annotation(
        &self,
        target: &ID2D1RenderTarget,
        a: &Annotation,
        ink: &ID2D1SolidColorBrush,
        white: &ID2D1SolidColorBrush,
    ) {
        let start = a.points[0];
        let end = *a.points.last().unwrap();
        match a.tool {
            Tool::Pen => {
                for pair in a.points.windows(2) {
                    target.DrawLine(point(pair[0]), point(pair[1]), ink, a.width, None);
                }
                for p in &a.points {
                    target.FillEllipse(
                        &D2D1_ELLIPSE {
                            point: point(*p),
                            radiusX: a.width / 2.,
                            radiusY: a.width / 2.,
                        },
                        ink,
                    );
                }
            }
            Tool::Arrow => {
                target.DrawLine(point(start), point(end), ink, a.width, None);
                let angle = (end.y - start.y).atan2(end.x - start.x);
                let length = (15. * self.scale).min(start.distance(end) * 0.45);
                for offset in [-0.5_f32, 0.5] {
                    target.DrawLine(
                        point(end),
                        point(Point::new(
                            end.x - length * (angle + offset).cos(),
                            end.y - length * (angle + offset).sin(),
                        )),
                        ink,
                        a.width,
                        None,
                    );
                }
            }
            Tool::Rectangle => {
                target.DrawRectangle(&rect(Rect::between(start, end)), ink, a.width, None)
            }
            Tool::Number => {
                let radius = 13. * self.scale;
                target.FillEllipse(
                    &D2D1_ELLIPSE {
                        point: point(start),
                        radiusX: radius,
                        radiusY: radius,
                    },
                    ink,
                );
                self.text(
                    target,
                    &a.number.to_string(),
                    Rect::between(
                        Point::new(start.x - radius, start.y - radius),
                        Point::new(start.x + radius, start.y + radius),
                    ),
                    white,
                    true,
                );
            }
        }
    }
    pub unsafe fn paint(
        &self,
        doc: &Document,
        full: Rect,
        selecting: Option<Rect>,
        toolbar: Option<Rect>,
        hint: &str,
    ) -> Result<()> {
        let target: ID2D1RenderTarget = self.target.cast()?;
        target.BeginDraw();
        let result = (|| {
            self.image(&target, &self.bitmap, doc, full)?;
            let shade = target.CreateSolidColorBrush(&color(0.025, 0.035, 0.065, 0.52), None)?;
            let accent = target.CreateSolidColorBrush(&color(0.55, 0.67, 1., 1.), None)?;
            let white = target.CreateSolidColorBrush(&color(0.94, 0.95, 0.98, 1.), None)?;
            let panel = target.CreateSolidColorBrush(&color(0.065, 0.075, 0.105, 1.), None)?;
            if let Some(r) = doc.selection.or(selecting) {
                for outside in [
                    Rect {
                        bottom: r.top,
                        ..full
                    },
                    Rect {
                        top: r.bottom,
                        ..full
                    },
                    Rect {
                        top: r.top,
                        bottom: r.bottom,
                        right: r.left,
                        ..full
                    },
                    Rect {
                        top: r.top,
                        bottom: r.bottom,
                        left: r.right,
                        ..full
                    },
                ] {
                    target.FillRectangle(&rect(outside), &shade);
                }
                target.DrawRectangle(&rect(r), &accent, self.scale, None);
                for p in [
                    Point::new(r.left, r.top),
                    Point::new(r.right, r.top),
                    Point::new(r.left, r.bottom),
                    Point::new(r.right, r.bottom),
                ] {
                    let s = 3. * self.scale;
                    target.FillRectangle(
                        &rect(Rect::between(
                            Point::new(p.x - s, p.y - s),
                            Point::new(p.x + s, p.y + s),
                        )),
                        &accent,
                    );
                }
            } else {
                target.FillRectangle(&rect(full), &shade);
            }
            if let Some(bar) = toolbar {
                target.FillRoundedRectangle(
                    &D2D1_ROUNDED_RECT {
                        rect: rect(bar),
                        radiusX: 9. * self.scale,
                        radiusY: 9. * self.scale,
                    },
                    &panel,
                );
                let labels = [
                    "1  Pen",
                    "2  Arrow",
                    "3  Box",
                    "4  Number",
                    "Copy ↵",
                    "Cancel",
                ];
                for (index, label) in labels.iter().enumerate() {
                    let cell = Rect {
                        left: bar.left + index as f32 * 92. * self.scale,
                        right: bar.left + (index + 1) as f32 * 92. * self.scale,
                        ..bar
                    };
                    if index < 4 && doc.tool == Tool::ALL[index] {
                        target.FillRoundedRectangle(
                            &D2D1_ROUNDED_RECT {
                                rect: rect(Rect {
                                    left: cell.left + 4. * self.scale,
                                    right: cell.right - 4. * self.scale,
                                    top: cell.top + 4. * self.scale,
                                    bottom: cell.bottom - 4. * self.scale,
                                }),
                                radiusX: 5. * self.scale,
                                radiusY: 5. * self.scale,
                            },
                            &accent,
                        );
                    }
                    self.text(
                        &target,
                        label,
                        Rect {
                            left: cell.left + 10. * self.scale,
                            ..cell
                        },
                        if index < 4 && doc.tool == Tool::ALL[index] {
                            &panel
                        } else {
                            &white
                        },
                        false,
                    );
                }
            }
            if !hint.is_empty() {
                // Position inside the current monitor, rather than the virtual desktop center.
                let (x, y) = if let Some(bar) = toolbar {
                    (
                        bar.left,
                        (bar.bottom + 8. * self.scale).min(full.bottom - 36. * self.scale),
                    )
                } else {
                    (24. * self.scale, 24. * self.scale)
                };
                let area = Rect {
                    left: x,
                    top: y,
                    right: (x + 740. * self.scale).min(full.right - 8.),
                    bottom: y + 32. * self.scale,
                };
                target.FillRoundedRectangle(
                    &D2D1_ROUNDED_RECT {
                        rect: rect(area),
                        radiusX: 6. * self.scale,
                        radiusY: 6. * self.scale,
                    },
                    &panel,
                );
                self.text(
                    &target,
                    hint,
                    Rect {
                        left: area.left + 12. * self.scale,
                        ..area
                    },
                    &white,
                    false,
                );
            }
            Ok(())
        })();
        let end = target.EndDraw(None, None);
        result.and(end)
    }
    pub unsafe fn export(
        &self,
        doc: &Document,
        pixels: &[u8],
        full: Rect,
    ) -> Result<super::capture::Surface> {
        let selection = doc
            .selection
            .ok_or_else(|| Error::new(E_INVALIDARG, "Select an image first"))?;
        let surface =
            super::capture::Surface::new(selection.width() as i32, selection.height() as i32)?;
        let target = self.factory.CreateDCRenderTarget(&properties())?;
        target.BindDC(
            surface.dc,
            &RECT {
                left: 0,
                top: 0,
                right: surface.width,
                bottom: surface.height,
            },
        )?;
        let target: ID2D1RenderTarget = target.cast()?;
        let bitmap = target.CreateBitmap(
            D2D_SIZE_U {
                width: full.width() as u32,
                height: full.height() as u32,
            },
            Some(pixels.as_ptr().cast()),
            full.width() as u32 * 4,
            &D2D1_BITMAP_PROPERTIES {
                pixelFormat: properties().pixelFormat,
                dpiX: 96.,
                dpiY: 96.,
            },
        )?;
        target.SetTransform(&Matrix3x2 {
            M11: 1.,
            M12: 0.,
            M21: 0.,
            M22: 1.,
            M31: -selection.left,
            M32: -selection.top,
        });
        target.BeginDraw();
        let result = self.image(&target, &bitmap, doc, full);
        let end = target.EndDraw(None, None);
        result.and(end)?;
        Ok(surface)
    }
}
