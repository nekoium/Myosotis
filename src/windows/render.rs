use crate::{
    document::{Annotation, Document, Point, Rect, Tool},
    settings::Theme,
};
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

use std::{cell::RefCell, collections::HashMap};

const ANNOTATION_INK: D2D1_COLOR_F = D2D1_COLOR_F {
    r: 199. / 255.,
    g: 70. / 255.,
    b: 86. / 255.,
    a: 1.,
};
const INTERFACE_ACCENT: D2D1_COLOR_F = D2D1_COLOR_F {
    r: 88. / 255.,
    g: 101. / 255.,
    b: 216. / 255.,
    a: 1.,
};

struct NumberBadge {
    layout: IDWriteTextLayout,
    offset: Point,
    half_width: f32,
    radius: f32,
}
impl NumberBadge {
    unsafe fn format(write: &IDWriteFactory, scale: f32) -> Result<IDWriteTextFormat> {
        let format = write.CreateTextFormat(
            w!("Segoe UI"),
            None,
            DWRITE_FONT_WEIGHT_SEMI_BOLD,
            DWRITE_FONT_STYLE_NORMAL,
            DWRITE_FONT_STRETCH_NORMAL,
            14. * scale,
            w!("en-us"),
        )?;
        format.SetWordWrapping(DWRITE_WORD_WRAPPING_NO_WRAP)?;
        Ok(format)
    }

    unsafe fn new(
        write: &IDWriteFactory,
        format: &IDWriteTextFormat,
        number: usize,
        scale: f32,
    ) -> Result<Self> {
        let text: Vec<u16> = number.to_string().encode_utf16().collect();
        let width = 256. * scale;
        let height = 64. * scale;
        let layout = write.CreateTextLayout(&text, format, width, height)?;
        // Overhangs are relative to the layout box, including its unused space.
        let bounds = layout.GetOverhangMetrics()?;
        let left = -bounds.left;
        let top = -bounds.top;
        let right = width + bounds.right;
        let bottom = height + bounds.bottom;
        let radius = 12. * scale;
        // Optical corrections for Segoe UI Semibold's asymmetric single-digit ink weight.
        let optical = match number {
            1 => Point::new(-0.5, 0.25),
            2 => Point::new(0., -0.25),
            3 => Point::new(-0.25, 0.),
            4 => Point::new(-0.25, -0.25),
            5 | 9 => Point::new(0., 0.25),
            7 => Point::new(0., 1.),
            _ => Point::default(),
        };
        Ok(Self {
            layout,
            offset: Point::new(
                -(left + right) / 2. + optical.x * scale,
                -(top + bottom) / 2. + optical.y * scale,
            ),
            half_width: radius.max((right - left) / 2. + 8. * scale),
            radius,
        })
    }

    unsafe fn draw(
        &self,
        target: &ID2D1RenderTarget,
        center: Point,
        ink: &ID2D1SolidColorBrush,
        white: &ID2D1SolidColorBrush,
    ) {
        target.FillRoundedRectangle(
            &D2D1_ROUNDED_RECT {
                rect: rect(Rect::between(
                    Point::new(center.x - self.half_width, center.y - self.radius),
                    Point::new(center.x + self.half_width, center.y + self.radius),
                )),
                radiusX: self.radius,
                radiusY: self.radius,
            },
            ink,
        );
        target.DrawTextLayout(
            point(Point::new(
                center.x + self.offset.x,
                center.y + self.offset.y,
            )),
            &self.layout,
            white,
            D2D1_DRAW_TEXT_OPTIONS_NO_SNAP,
        );
    }
}

struct InkCache {
    target: ID2D1BitmapRenderTarget,
    epoch: u64,
    completed: usize,
    active: Option<u64>,
    segments: usize,
}

pub const TOOLBAR_CELL: f32 = 40.;
pub const TOOLBAR_HEIGHT: f32 = 44.;

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
    pub theme: Theme,
    factory: ID2D1Factory,
    pub target: ID2D1HwndRenderTarget,
    bitmap: ID2D1Bitmap,
    small: IDWriteTextFormat,
    number: IDWriteTextFormat,
    write: IDWriteFactory,
    numbers: RefCell<HashMap<usize, NumberBadge>>,
    scale: f32,
    rounded_stroke: ID2D1StrokeStyle,
    ink: RefCell<Option<InkCache>>,
}
impl Renderer {
    pub unsafe fn new(
        hwnd: HWND,
        width: u32,
        height: u32,
        pixels: &[u8],
        scale: f32,
        theme: Theme,
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
        let number = NumberBadge::format(&write, scale)?;
        let rounded_stroke = factory.CreateStrokeStyle(
            &D2D1_STROKE_STYLE_PROPERTIES {
                startCap: D2D1_CAP_STYLE_ROUND,
                endCap: D2D1_CAP_STYLE_ROUND,
                lineJoin: D2D1_LINE_JOIN_ROUND,
                ..Default::default()
            },
            None,
        )?;
        Ok(Self {
            theme,
            factory,
            target,
            bitmap,
            small,
            number,
            write,
            numbers: RefCell::new(HashMap::new()),
            scale,
            rounded_stroke,
            ink: RefCell::new(None),
        })
    }
    unsafe fn text(
        &self,
        target: &ID2D1RenderTarget,
        text: &str,
        bounds: Rect,
        brush: &ID2D1SolidColorBrush,
    ) {
        let text: Vec<u16> = text.encode_utf16().collect();
        target.DrawText(
            &text,
            &self.small,
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
        cached: bool,
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
            let ink = target.CreateSolidColorBrush(&ANNOTATION_INK, None)?;
            let result: Result<()> = (|| {
                if cached {
                    let cache = self.ink.borrow();
                    let cache = cache.as_ref().unwrap();
                    let bitmap = cache.target.GetBitmap()?;
                    target.DrawBitmap(
                        &bitmap,
                        Some(&rect(full)),
                        1.,
                        D2D1_BITMAP_INTERPOLATION_MODE_NEAREST_NEIGHBOR,
                        None,
                    );
                    if let Some(a) = &doc.active {
                        if a.tool == Tool::Pen {
                            self.pen(
                                target,
                                a,
                                cache.segments,
                                a.points.len().saturating_sub(1),
                                true,
                                &ink,
                            )?;
                        } else {
                            self.annotation(target, a, &ink, &white)?;
                        }
                    }
                } else {
                    for a in doc.annotations.iter().chain(doc.active.iter()) {
                        self.annotation(target, a, &ink, &white)?;
                    }
                }
                Ok(())
            })();
            target.PopAxisAlignedClip();
            result?;
        }
        Ok(())
    }
    unsafe fn pen(
        &self,
        target: &ID2D1RenderTarget,
        a: &Annotation,
        first: usize,
        end: usize,
        taper_end: bool,
        ink: &ID2D1SolidColorBrush,
    ) -> Result<()> {
        let stroke = a.stroke.as_ref().unwrap();
        let disk = |n: crate::stroke::Node| {
            target.FillEllipse(
                &D2D1_ELLIPSE {
                    point: point(n.point),
                    radiusX: n.radius,
                    radiusY: n.radius,
                },
                ink,
            );
        };
        for i in first..end {
            let nodes = stroke.segment(i, taper_end);
            for pair in nodes.windows(2) {
                let a = pair[0];
                let b = pair[1];
                if (a.radius - b.radius).abs() < 0.001 {
                    target.DrawLine(point(a.point), point(b.point), ink, a.radius * 2., None);
                } else {
                    let dx = b.point.x - a.point.x;
                    let dy = b.point.y - a.point.y;
                    let length = dx.hypot(dy);
                    // External tangents keep a changing radius from making segment corners protrude.
                    let slope = ((a.radius - b.radius) / length).clamp(-1., 1.);
                    let normal = (1. - slope * slope).sqrt();
                    let offset = |p: Point, radius: f32, sign: f32| {
                        point(Point::new(
                            p.x + radius * (dx / length * slope - dy / length * normal * sign),
                            p.y + radius * (dy / length * slope + dx / length * normal * sign),
                        ))
                    };
                    let geometry = self.factory.CreatePathGeometry()?;
                    let sink = geometry.Open()?;
                    sink.BeginFigure(offset(a.point, a.radius, 1.), D2D1_FIGURE_BEGIN_FILLED);
                    sink.AddLines(&[
                        offset(a.point, a.radius, -1.),
                        offset(b.point, b.radius, -1.),
                        offset(b.point, b.radius, 1.),
                    ]);
                    sink.EndFigure(D2D1_FIGURE_END_CLOSED);
                    sink.Close()?;
                    target.FillGeometry(&geometry, ink, None);
                }
                // Flat segment ends avoid dark beads from overlapping round caps on straight ink.
                // Only the stroke ends and actual turns need round coverage.
                let corner = if i > 0 {
                    let p = stroke.nodes[i - 1].point;
                    let q = stroke.nodes[i].point;
                    let r = stroke.nodes[i + 1].point;
                    let ux = q.x - p.x;
                    let uy = q.y - p.y;
                    let vx = r.x - q.x;
                    let vy = r.y - q.y;
                    (ux * vy - uy * vx).abs() > 0.001 || ux * vx + uy * vy < 0.
                } else {
                    true
                };
                if corner || a.distance != stroke.nodes[i].distance {
                    disk(a);
                }
            }
        }
        if taper_end && (end == stroke.nodes.len() - 1) {
            disk(stroke.disk(end, taper_end));
        }
        Ok(())
    }
    unsafe fn update_ink(&self, doc: &Document) -> Result<()> {
        let mut slot = self.ink.borrow_mut();
        if slot.is_none() {
            let format = D2D1_PIXEL_FORMAT {
                format: DXGI_FORMAT_B8G8R8A8_UNORM,
                alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
            };
            let target = self.target.CreateCompatibleRenderTarget(
                None,
                None,
                Some(&format),
                D2D1_COMPATIBLE_RENDER_TARGET_OPTIONS_NONE,
            )?;
            *slot = Some(InkCache {
                target,
                epoch: u64::MAX,
                completed: 0,
                active: None,
                segments: 0,
            });
        }
        let cache = slot.as_mut().unwrap();
        let abandoned = cache.active.is_some_and(|id| {
            doc.active.as_ref().is_none_or(|a| a.id != id)
                && doc
                    .annotations
                    .get(cache.completed)
                    .is_none_or(|a| a.id != id)
        });
        let reset =
            cache.epoch != doc.cache_epoch || cache.completed > doc.annotations.len() || abandoned;
        let target: ID2D1RenderTarget = cache.target.cast()?;
        target.BeginDraw();
        let result = (|| {
            if reset {
                target.Clear(Some(&color(0., 0., 0., 0.)));
                cache.completed = 0;
                cache.active = None;
                cache.segments = 0;
                cache.epoch = doc.cache_epoch;
            }
            let ink = target.CreateSolidColorBrush(&ANNOTATION_INK, None)?;
            let white = target.CreateSolidColorBrush(&color(1., 1., 1., 1.), None)?;
            for a in &doc.annotations[cache.completed..] {
                if cache.active == Some(a.id) {
                    self.pen(
                        &target,
                        a,
                        cache.segments,
                        a.points.len().saturating_sub(1),
                        true,
                        &ink,
                    )?;
                } else {
                    self.annotation(&target, a, &ink, &white)?;
                }
                cache.completed += 1;
                cache.active = None;
                cache.segments = 0;
            }
            if let Some(a) = doc.active.as_ref().filter(|a| a.tool == Tool::Pen) {
                cache.active = Some(a.id);
                let end = a.stroke.as_ref().unwrap().stable_segments();
                if end > cache.segments {
                    self.pen(&target, a, cache.segments, end, false, &ink)?;
                    cache.segments = end;
                }
            }
            Ok(())
        })();
        let end = target.EndDraw(None, None);
        let result = result.and(end);
        if result.is_err() {
            *slot = None;
        }
        result
    }
    unsafe fn annotation(
        &self,
        target: &ID2D1RenderTarget,
        a: &Annotation,
        ink: &ID2D1SolidColorBrush,
        white: &ID2D1SolidColorBrush,
    ) -> Result<()> {
        let start = a.points[0];
        let end = *a.points.last().unwrap();
        match a.tool {
            Tool::Pen => self.pen(target, a, 0, a.points.len().saturating_sub(1), true, ink)?,
            Tool::Arrow => {
                target.DrawLine(
                    point(start),
                    point(end),
                    ink,
                    a.width * 0.8,
                    Some(&self.rounded_stroke),
                );
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
                        a.width * 0.8,
                        Some(&self.rounded_stroke),
                    );
                }
            }
            Tool::Rectangle => target.DrawRoundedRectangle(
                &D2D1_ROUNDED_RECT {
                    rect: rect(Rect::between(start, end)),
                    radiusX: 3. * self.scale,
                    radiusY: 3. * self.scale,
                },
                ink,
                a.width * 0.6,
                None,
            ),
            Tool::Number => {
                let mut numbers = self.numbers.borrow_mut();
                if let std::collections::hash_map::Entry::Vacant(entry) = numbers.entry(a.number) {
                    entry.insert(NumberBadge::new(
                        &self.write,
                        &self.number,
                        a.number,
                        self.scale,
                    )?);
                }
                numbers[&a.number].draw(target, start, ink, white);
            }
        }
        Ok(())
    }
    unsafe fn icon(
        &self,
        target: &ID2D1RenderTarget,
        index: usize,
        cell: Rect,
        brush: &ID2D1SolidColorBrush,
        size: f32,
    ) -> Result<()> {
        let size = size * self.scale;
        let x = (cell.left + cell.right - size) / 2.;
        let y = (cell.top + cell.bottom - size) / 2.;
        let p = |a: f32, b: f32| D2D_POINT_2F {
            x: x + a * size / 24.,
            y: y + b * size / 24.,
        };
        let line = |a, b, c, d| {
            target.DrawLine(
                p(a, b),
                p(c, d),
                brush,
                1.8 * self.scale,
                Some(&self.rounded_stroke),
            );
        };
        let box_at = |a, b, c, d| {
            let start = p(a, b);
            let end = p(c, d);
            target.DrawRoundedRectangle(
                &D2D1_ROUNDED_RECT {
                    rect: D2D_RECT_F {
                        left: start.x,
                        top: start.y,
                        right: end.x,
                        bottom: end.y,
                    },
                    radiusX: 1.5 * self.scale,
                    radiusY: 1.5 * self.scale,
                },
                brush,
                1.8 * self.scale,
                None,
            );
        };
        match index {
            0 => {
                line(4., 16., 16., 4.);
                line(16., 4., 20., 8.);
                line(20., 8., 8., 20.);
                line(8., 20., 4., 20.);
                line(4., 20., 4., 16.);
                line(13., 7., 17., 11.);
            }
            1 => {
                line(5., 19., 19., 5.);
                line(9., 5., 19., 5.);
                line(19., 5., 19., 15.);
            }
            2 => box_at(4., 5., 20., 19.),
            3 => {
                target.DrawEllipse(
                    &D2D1_ELLIPSE {
                        point: p(12., 12.),
                        radiusX: size * 0.39,
                        radiusY: size * 0.39,
                    },
                    brush,
                    1.8 * self.scale,
                    None,
                );
                line(10., 9., 12., 7.);
                line(12., 7., 12., 17.);
                line(9., 17., 15., 17.);
            }
            4 => {
                box_at(8., 8., 20., 21.);
                line(16., 4., 4., 4.);
                line(4., 4., 4., 16.);
            }
            5 => {
                line(6., 6., 18., 18.);
                line(6., 18., 18., 6.);
            }
            6 | 7 => {
                let flip = |a: f32| if index == 7 { 24. - a } else { a };
                line(flip(8.), 3., flip(3.), 8.);
                line(flip(3.), 8., flip(8.), 13.);
                let path = self.factory.CreatePathGeometry()?;
                let sink = path.Open()?;
                sink.BeginFigure(p(flip(3.), 8.), D2D1_FIGURE_BEGIN_HOLLOW);
                sink.AddLine(p(flip(14.), 8.));
                sink.AddBezier(&D2D1_BEZIER_SEGMENT {
                    point1: p(flip(17.314), 8.),
                    point2: p(flip(20.), 10.686),
                    point3: p(flip(20.), 14.),
                });
                sink.AddBezier(&D2D1_BEZIER_SEGMENT {
                    point1: p(flip(20.), 17.314),
                    point2: p(flip(17.314), 20.),
                    point3: p(flip(14.), 20.),
                });
                sink.EndFigure(D2D1_FIGURE_END_OPEN);
                sink.Close()?;
                target.DrawGeometry(&path, brush, 1.8 * self.scale, Some(&self.rounded_stroke));
            }
            8 => {
                let path = self.factory.CreatePathGeometry()?;
                let sink = path.Open()?;
                sink.BeginFigure(p(19., 8.), D2D1_FIGURE_BEGIN_HOLLOW);
                sink.AddArc(&D2D1_ARC_SEGMENT {
                    point: p(20., 15.),
                    size: D2D_SIZE_F {
                        width: 8. * size / 24.,
                        height: 8. * size / 24.,
                    },
                    rotationAngle: 0.,
                    sweepDirection: D2D1_SWEEP_DIRECTION_COUNTER_CLOCKWISE,
                    arcSize: D2D1_ARC_SIZE_LARGE,
                });
                sink.EndFigure(D2D1_FIGURE_END_OPEN);
                sink.Close()?;
                target.DrawGeometry(&path, brush, 1.8 * self.scale, Some(&self.rounded_stroke));
                line(19., 3., 19., 8.);
                line(19., 8., 14., 8.);
            }
            10 => {
                let path = self.factory.CreatePathGeometry()?;
                let sink = path.Open()?;
                sink.BeginFigure(p(3., 9.), D2D1_FIGURE_BEGIN_HOLLOW);
                sink.AddBezier(&D2D1_BEZIER_SEGMENT {
                    point1: p(7., -1.),
                    point2: p(21., 2.),
                    point3: p(21., 12.),
                });
                sink.AddBezier(&D2D1_BEZIER_SEGMENT {
                    point1: p(21., 24.),
                    point2: p(4., 24.),
                    point3: p(4., 14.),
                });
                sink.EndFigure(D2D1_FIGURE_END_OPEN);
                sink.Close()?;
                target.DrawGeometry(&path, brush, 1.8 * self.scale, Some(&self.rounded_stroke));
                line(3., 3., 3., 9.);
                line(3., 9., 9., 9.);
                line(12., 8., 12., 13.);
                line(12., 13., 15., 15.);
            }
            9 => {
                for (a, b, c, d) in [
                    (4., 4., 7., 4.),
                    (11., 4., 14., 4.),
                    (20., 10., 20., 13.),
                    (20., 17., 20., 20.),
                    (17., 20., 20., 20.),
                    (10., 20., 13., 20.),
                    (4., 4., 4., 7.),
                    (4., 11., 4., 14.),
                ] {
                    line(a, b, c, d);
                }
                for (a, b) in [(18., 2.), (2., 18.)] {
                    let start = p(a, b);
                    let end = p(a + 4., b + 4.);
                    target.DrawRectangle(
                        &D2D_RECT_F {
                            left: start.x,
                            top: start.y,
                            right: end.x,
                            bottom: end.y,
                        },
                        brush,
                        1.4 * self.scale,
                        None,
                    );
                }
            }
            _ => {}
        }
        Ok(())
    }
    pub unsafe fn schema_keys(&self, bindings: &crate::input::Bindings) -> Result<[Rect; 10]> {
        let format = self.write.CreateTextFormat(
            w!("Segoe UI"),
            None,
            DWRITE_FONT_WEIGHT_NORMAL,
            DWRITE_FONT_STYLE_NORMAL,
            DWRITE_FONT_STRETCH_NORMAL,
            11. * self.scale,
            w!("en-us"),
        )?;
        let mut keys = [Rect::default(); 10];
        for (index, rect) in keys.iter_mut().enumerate() {
            let text: Vec<u16> = super::schema::label(bindings, index)
                .encode_utf16()
                .collect();
            let layout = self.write.CreateTextLayout(
                &text,
                &format,
                1000. * self.scale,
                40. * self.scale,
            )?;
            let mut metrics = DWRITE_TEXT_METRICS::default();
            layout.GetMetrics(&mut metrics)?;
            *rect = super::schema::key_rect(
                index,
                metrics.widthIncludingTrailingWhitespace / self.scale,
            );
        }
        Ok(keys)
    }
    pub unsafe fn paint_schema(&self, window: &super::schema::Schema) -> Result<()> {
        self.paint_schema_on(
            &self.target.cast()?,
            &window.bindings,
            window.focus,
            window.listening,
            window.hover,
        )
    }
    unsafe fn paint_schema_on(
        &self,
        target: &ID2D1RenderTarget,
        bindings: &crate::input::Bindings,
        focus: Option<usize>,
        listening: Option<usize>,
        hover: Option<usize>,
    ) -> Result<()> {
        use super::schema::{self, WIDTH};
        let (surface, foreground, key, selected, muted) = match self.theme {
            Theme::Dark => (
                color(0.14, 0.14, 0.15, 1.),
                color(0.94, 0.94, 0.95, 1.),
                color(0.22, 0.22, 0.24, 1.),
                color(0.32, 0.32, 0.35, 1.),
                color(0.65, 0.65, 0.69, 1.),
            ),
            Theme::Light => (
                color(0.94, 0.94, 0.95, 1.),
                color(0.12, 0.12, 0.14, 1.),
                color(0.86, 0.86, 0.88, 1.),
                color(0.76, 0.76, 0.79, 1.),
                color(0.40, 0.40, 0.44, 1.),
            ),
        };
        let fg = target.CreateSolidColorBrush(&foreground, None)?;
        let muted = target.CreateSolidColorBrush(&muted, None)?;
        let key_brush = target.CreateSolidColorBrush(&key, None)?;
        let selected_brush = target.CreateSolidColorBrush(&selected, None)?;
        let accent = target.CreateSolidColorBrush(&INTERFACE_ACCENT, None)?;
        let white = target.CreateSolidColorBrush(&color(1., 1., 1., 1.), None)?;
        let scaled = |r: Rect| {
            rect(Rect {
                left: r.left * self.scale,
                top: r.top * self.scale,
                right: r.right * self.scale,
                bottom: r.bottom * self.scale,
            })
        };
        let fill = |r: Rect, brush: &ID2D1SolidColorBrush, radius: f32| {
            target.FillRoundedRectangle(
                &D2D1_ROUNDED_RECT {
                    rect: scaled(r),
                    radiusX: radius * self.scale,
                    radiusY: radius * self.scale,
                },
                brush,
            )
        };
        let text =
            |value: &str, r: Rect, brush: &ID2D1SolidColorBrush, center: bool| -> Result<()> {
                let title = !center && r.top < 40.;
                let font_size = if title {
                    12.
                } else if center && r.width() < 60. && value.chars().count() > 7 {
                    9.
                } else {
                    11.
                };
                let value: Vec<u16> = value.encode_utf16().collect();
                let format = self.write.CreateTextFormat(
                    w!("Segoe UI"),
                    None,
                    if title {
                        DWRITE_FONT_WEIGHT_SEMI_BOLD
                    } else {
                        DWRITE_FONT_WEIGHT_NORMAL
                    },
                    DWRITE_FONT_STYLE_NORMAL,
                    DWRITE_FONT_STRETCH_NORMAL,
                    font_size * self.scale,
                    w!("en-us"),
                )?;
                format.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER)?;
                if center {
                    format.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER)?;
                }
                target.DrawText(
                    &value,
                    &format,
                    &scaled(r),
                    brush,
                    D2D1_DRAW_TEXT_OPTIONS_CLIP,
                    DWRITE_MEASURING_MODE_NATURAL,
                );
                Ok(())
            };
        let keys = self.schema_keys(bindings)?;
        target.BeginDraw();
        target.Clear(Some(&surface));
        text(
            "Input schema",
            Rect {
                left: 40.,
                top: 10.,
                right: 200.,
                bottom: 40.,
            },
            &fg,
            false,
        )?;
        // Reproduce the existing assets/myosotis.svg mark without another image resource.
        for (degrees, rgb) in [
            (0., (100., 141., 240.)),
            (72., (126., 163., 248.)),
            (144., (107., 149., 242.)),
            (216., (134., 170., 247.)),
            (288., (115., 156., 244.)),
        ] {
            let angle = (degrees as f32).to_radians();
            let p = |x: f32, y: f32| {
                point(Point::new(
                    (22. + (x * angle.cos() - y * angle.sin()) * 1.16 * 18. / 256.) * self.scale,
                    (25. + (x * angle.sin() + y * angle.cos()) * 1.16 * 18. / 256.) * self.scale,
                ))
            };
            let petal = self.factory.CreatePathGeometry()?;
            let sink = petal.Open()?;
            sink.BeginFigure(p(-13., -4.), D2D1_FIGURE_BEGIN_FILLED);
            for (a, b, c, d, e, f) in [
                (-48., -28., -51., -62., -34., -87.),
                (-13., -115., 35., -103., 41., -76.),
                (49., -48., 28., -12., -13., -4.),
            ] {
                sink.AddBezier(&D2D1_BEZIER_SEGMENT {
                    point1: p(a, b),
                    point2: p(c, d),
                    point3: p(e, f),
                });
            }
            sink.EndFigure(D2D1_FIGURE_END_CLOSED);
            sink.Close()?;
            let brush = target.CreateSolidColorBrush(
                &color(rgb.0 / 255., rgb.1 / 255., rgb.2 / 255., 1.),
                None,
            )?;
            target.FillGeometry(&petal, &brush, None);
        }
        let center =
            target.CreateSolidColorBrush(&color(245. / 255., 200. / 255., 92. / 255., 1.), None)?;
        target.FillEllipse(
            &D2D1_ELLIPSE {
                point: point(Point::new(22. * self.scale, 25. * self.scale)),
                radiusX: 23. * 1.16 * 18. / 256. * self.scale,
                radiusY: 23. * 1.16 * 18. / 256. * self.scale,
            },
            &center,
        );
        for (index, r) in [(10, schema::reset_rect()), (11, schema::close_rect())] {
            let enabled = index == 11 || bindings != &crate::input::Bindings::default();
            if enabled && (hover == Some(index) || focus == Some(index)) {
                fill(r, &key_brush, 4.);
            }
            let r = Rect {
                left: r.left * self.scale,
                top: r.top * self.scale,
                right: r.right * self.scale,
                bottom: r.bottom * self.scale,
            };
            self.icon(
                target,
                if index == 10 { 10 } else { 5 },
                r,
                if enabled { &fg } else { &muted },
                14.,
            )?;
        }
        for (index, &key_rect) in keys.iter().enumerate() {
            let r = schema::icon_rect(index);
            if index == 1 {
                fill(r, &selected_brush, 6.);
            }
            if index == 5 {
                fill(r, &accent, 6.);
            }
            let r = Rect {
                left: r.left * self.scale,
                top: r.top * self.scale,
                right: r.right * self.scale,
                bottom: r.bottom * self.scale,
            };
            self.icon(
                target,
                if index == 0 { 9 } else { index - 1 },
                r,
                if index == 5 { &white } else { &fg },
                20.,
            )?;
            let value = if listening == Some(index) {
                "…".into()
            } else if index == 0 {
                bindings.capture.label()
            } else {
                bindings.label(crate::input::Action::ALL[index - 1])
            };
            let r = key_rect;
            fill(
                r,
                if listening == Some(index) {
                    &accent
                } else if hover == Some(index) || focus == Some(index) {
                    &selected_brush
                } else {
                    &key_brush
                },
                4.,
            );
            text(
                &value,
                r,
                if listening == Some(index) {
                    &white
                } else {
                    &fg
                },
                true,
            )?;
        }
        for y in [94., 192.] {
            target.DrawLine(
                point(Point::new(16. * self.scale, y * self.scale)),
                point(Point::new((WIDTH - 16.) * self.scale, y * self.scale)),
                &key_brush,
                self.scale,
                None,
            );
        }
        target.DrawLine(
            point(Point::new(224. * self.scale, 116. * self.scale)),
            point(Point::new(224. * self.scale, 130. * self.scale)),
            &key_brush,
            self.scale,
            None,
        );
        target.EndDraw(None, None)
    }
    pub unsafe fn paint(
        &self,
        doc: &Document,
        full: Rect,
        selecting: Option<Rect>,
        toolbar: Option<Rect>,
        pointer: Point,
        hint: &str,
    ) -> Result<()> {
        self.update_ink(doc)?;
        let target: ID2D1RenderTarget = self.target.cast()?;
        target.BeginDraw();
        let result = (|| {
            self.image(&target, &self.bitmap, doc, full, true)?;
            let shade = target.CreateSolidColorBrush(&color(0.02, 0.02, 0.02, 0.38), None)?;
            let accent = target.CreateSolidColorBrush(&INTERFACE_ACCENT, None)?;
            let white = target.CreateSolidColorBrush(&color(0.94, 0.94, 0.95, 1.), None)?;
            let (panel_color, foreground_color, hover_color, selected_color) = match self.theme {
                Theme::Dark => (
                    color(0.14, 0.14, 0.15, 1.),
                    color(0.94, 0.94, 0.95, 1.),
                    color(0.22, 0.22, 0.24, 1.),
                    color(0.32, 0.32, 0.35, 1.),
                ),
                Theme::Light => (
                    color(0.94, 0.94, 0.95, 1.),
                    color(0.12, 0.12, 0.14, 1.),
                    color(0.86, 0.86, 0.88, 1.),
                    color(0.76, 0.76, 0.79, 1.),
                ),
            };
            let panel = target.CreateSolidColorBrush(&panel_color, None)?;
            let foreground = target.CreateSolidColorBrush(&foreground_color, None)?;
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
                        radiusX: 10. * self.scale,
                        radiusY: 10. * self.scale,
                    },
                    &panel,
                );
                let hover = target.CreateSolidColorBrush(&hover_color, None)?;
                let selected_fill = target.CreateSolidColorBrush(&selected_color, None)?;
                for index in 0..6 {
                    let cell = Rect {
                        left: bar.left + index as f32 * TOOLBAR_CELL * self.scale,
                        right: bar.left + (index + 1) as f32 * TOOLBAR_CELL * self.scale,
                        ..bar
                    };
                    let selected = index < 4 && doc.tool == Tool::ALL[index];
                    if selected || index == 4 || cell.contains(pointer) {
                        target.FillRoundedRectangle(
                            &D2D1_ROUNDED_RECT {
                                rect: rect(Rect {
                                    left: cell.left + 4. * self.scale,
                                    right: cell.right - 4. * self.scale,
                                    top: cell.top + 6. * self.scale,
                                    bottom: cell.bottom - 6. * self.scale,
                                }),
                                radiusX: 6. * self.scale,
                                radiusY: 6. * self.scale,
                            },
                            if index == 4 {
                                &accent
                            } else if selected {
                                &selected_fill
                            } else {
                                &hover
                            },
                        );
                    }
                    if index == 4 {
                        target.DrawLine(
                            point(Point::new(cell.left, bar.top + 16. * self.scale)),
                            point(Point::new(cell.left, bar.bottom - 16. * self.scale)),
                            &hover,
                            self.scale,
                            None,
                        );
                    }
                    self.icon(
                        &target,
                        index,
                        cell,
                        if index == 4 { &white } else { &foreground },
                        20.,
                    )?;
                }
            }
            if !hint.is_empty() {
                // Position inside the current monitor, rather than the virtual desktop center.
                let (x, y) = if let Some(bar) = toolbar {
                    (
                        bar.left,
                        if bar.bottom + 40. * self.scale <= full.bottom {
                            bar.bottom + 8. * self.scale
                        } else {
                            (bar.top - 40. * self.scale).max(full.top)
                        },
                    )
                } else {
                    (24. * self.scale, 24. * self.scale)
                };
                let width = (hint.chars().count() as f32 * 7.5 + 24.) * self.scale;
                let width = width.min((full.width() - 16. * self.scale).max(self.scale));
                let x = x.min(full.right - width - 8. * self.scale).max(full.left);
                let area = Rect {
                    left: x,
                    top: y,
                    right: x + width,
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
                    &foreground,
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
        let result = self.image(&target, &bitmap, doc, full, false);
        let end = target.EndDraw(None, None);
        result.and(end)?;
        Ok(surface)
    }
}

#[cfg(test)]
#[path = "number_badge_tests.rs"]
mod number_badge_tests;
#[cfg(test)]
#[path = "render_tests.rs"]
mod render_tests;
