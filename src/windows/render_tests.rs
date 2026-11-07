//! Native renderer checks with an invisible window: no screen capture, input, or clipboard.
use super::*;
use ::windows::Win32::UI::WindowsAndMessaging::*;
use std::time::Instant;

struct HiddenWindow(HWND);
impl Drop for HiddenWindow {
    fn drop(&mut self) {
        unsafe {
            let _ = DestroyWindow(self.0);
        }
    }
}

unsafe fn renderer() -> Result<(HiddenWindow, Renderer)> {
    // No WS_VISIBLE or activation: the user's desktop and resident app stay untouched.
    let window = HiddenWindow(CreateWindowExW(
        WINDOW_EX_STYLE::default(),
        w!("STATIC"),
        w!("Myosotis offscreen tests"),
        WS_POPUP,
        0,
        0,
        640,
        480,
        None,
        None,
        None,
        None,
    )?);
    let renderer = Renderer::new(
        window.0,
        640,
        480,
        &vec![255; 640 * 480 * 4],
        1.,
        Theme::Dark,
    )?;
    Ok((window, renderer))
}

fn document(strokes: usize) -> Document {
    let mut doc = Document::default();
    doc.select(
        Rect::between(Point::default(), Point::new(640., 480.)),
        true,
    );
    for row in 0..strokes {
        doc.begin_sample(Point::new(20., 30. + (row % 20) as f32 * 20.), 1., None, 0);
        for i in 1..=80 {
            doc.update_sample(
                Point::new(
                    20. + i as f32 * 7.,
                    30. + (row % 20) as f32 * 20. + (i as f32 * 0.2).sin() * 8.,
                ),
                None,
                i * 4,
            );
        }
        doc.finish();
    }
    doc
}

#[test]
fn offscreen_cache_preserves_history_and_discards_abandoned_ink() -> Result<()> {
    unsafe {
        let (_window, renderer) = renderer()?;
        let mut doc = document(25);
        renderer.update_ink(&doc)?;
        let bitmap = renderer.ink.borrow().as_ref().unwrap().target.clone();
        assert_eq!(renderer.ink.borrow().as_ref().unwrap().completed, 25);
        doc.begin_sample(Point::new(20., 240.), 1., None, 0);
        for i in 1..=640 {
            doc.update_sample(
                Point::new(20. + i as f32 * 0.8, 240. + (i as f32 * 0.1).sin() * 40.),
                None,
                i * 4,
            );
            renderer.update_ink(&doc)?;
            let slot = renderer.ink.borrow();
            let cache = slot.as_ref().unwrap();
            assert_eq!(
                cache.target, bitmap,
                "ordinary samples recreated the ink bitmap"
            );
            assert_eq!(cache.completed, 25);
            assert_eq!(cache.active, Some(doc.active.as_ref().unwrap().id));
            assert_eq!(
                cache.segments,
                doc.active
                    .as_ref()
                    .unwrap()
                    .stroke
                    .as_ref()
                    .unwrap()
                    .stable_segments()
            );
        }
        // Completion rollback must remove cached active ink without deleting earlier marks.
        doc.active = None;
        renderer.update_ink(&doc)?;
        assert!(renderer.ink.borrow().as_ref().unwrap().active.is_none());
        assert_eq!(renderer.ink.borrow().as_ref().unwrap().completed, 25);
        doc.undo();
        renderer.update_ink(&doc)?;
        assert_eq!(renderer.ink.borrow().as_ref().unwrap().completed, 24);
        doc.redo();
        renderer.update_ink(&doc)?;
        assert_eq!(renderer.ink.borrow().as_ref().unwrap().completed, 25);
        doc.reset();
        renderer.update_ink(&doc)?;
        assert_eq!(renderer.ink.borrow().as_ref().unwrap().completed, 0);
        doc.undo();
        renderer.update_ink(&doc)?;
        assert_eq!(renderer.ink.borrow().as_ref().unwrap().completed, 25);
        // Export to a memory bitmap, never the system clipboard.
        let surface = renderer.export(&doc, &vec![255; 640 * 480 * 4], doc.selection.unwrap())?;
        assert!(surface
            .pixels()
            .as_chunks::<4>()
            .0
            .iter()
            .any(|p| p[..3] == [86, 70, 199]));
        Ok(())
    }
}

#[test]
fn schema_renders_both_themes_and_recording_at_common_dpi_without_desktop_input() -> Result<()> {
    unsafe {
        let (_window, mut renderer) = renderer()?;
        let factory = renderer.factory.clone();
        for scale in [1., 1.25, 1.5, 2.] {
            renderer.scale = scale;
            let width = (super::super::schema::WIDTH * scale) as i32;
            let height = ((super::super::schema::HEIGHT + 58.) * scale) as i32;
            let surface = super::super::capture::Surface::new(width, height)?;
            let dc = factory.CreateDCRenderTarget(&properties())?;
            dc.BindDC(
                surface.dc,
                &RECT {
                    left: 0,
                    top: 0,
                    right: width,
                    bottom: height,
                },
            )?;
            let target: ID2D1RenderTarget = dc.cast()?;
            for theme in [Theme::Dark, Theme::Light] {
                renderer.theme = theme;
                let bindings = crate::input::Bindings::default();
                let keys = renderer.schema_keys(&bindings)?;
                assert!(
                    (keys[0].width() - 77.).abs() < 2.,
                    "Capture keycap must fit the reference's measured text width"
                );
                assert!(
                    (keys[5].width() - 32.).abs() < 2.,
                    "Ctrl keycap must not use character-count padding"
                );
                assert_eq!(keys[7].top, 248.);
                assert_eq!(super::super::schema::HEIGHT - keys[7].bottom, 21.);
                renderer.paint_schema_on(&target, &bindings, None, None, None)?;
                let pixel = |x: f32, y: f32| {
                    let index =
                        (((y * scale) as usize) * width as usize + (x * scale) as usize) * 4;
                    &surface.pixels()[index..index + 3]
                };
                assert_eq!(
                    pixel(239., 110.),
                    &[216, 101, 88],
                    "missing blue Copy control"
                );
                let bg = pixel(10., 100.);
                assert!(if theme == Theme::Dark {
                    bg[0] < 50
                } else {
                    bg[0] > 230
                });
                // Optional memory-only review artifact, never a desktop screenshot.
                if scale == 2. && std::env::var_os("MYOSOTIS_SCHEMA_PREVIEW").is_some() {
                    let cropped_height = (super::super::schema::HEIGHT * scale) as i32;
                    let pixels = &surface.pixels()[..(width * cropped_height * 4) as usize];
                    let mut bmp = vec![0u8; 54];
                    bmp[..2].copy_from_slice(b"BM");
                    bmp[2..6].copy_from_slice(&((54 + pixels.len()) as u32).to_le_bytes());
                    bmp[10..14].copy_from_slice(&54u32.to_le_bytes());
                    bmp[14..18].copy_from_slice(&40u32.to_le_bytes());
                    bmp[18..22].copy_from_slice(&width.to_le_bytes());
                    bmp[22..26].copy_from_slice(&(-cropped_height).to_le_bytes());
                    bmp[26..28].copy_from_slice(&1u16.to_le_bytes());
                    bmp[28..30].copy_from_slice(&32u16.to_le_bytes());
                    bmp.extend_from_slice(pixels);
                    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(".tmp");
                    std::fs::create_dir_all(&directory).unwrap();
                    std::fs::write(directory.join(format!("schema-{theme:?}.bmp")), bmp).unwrap();
                }
                for index in [1, 10, 11] {
                    let mut changed = bindings.clone();
                    changed.set(crate::input::Action::Pen, b'Q' as u16).unwrap();
                    renderer.paint_schema_on(&target, &changed, None, None, Some(index))?;
                    let hovered = surface.pixels().to_vec();
                    renderer.paint_schema_on(&target, &changed, Some(index), None, None)?;
                    assert_eq!(
                        surface.pixels(),
                        hovered,
                        "focus must use the neutral hover fill without an extra outline"
                    );
                }
                renderer.paint_schema_on(&target, &bindings, Some(1), Some(1), Some(1))?;
                assert_eq!(pixel(39., 155.), &[216, 101, 88], "missing recording state");
            }
        }
        Ok(())
    }
}

#[test]
#[ignore = "optional cursor-free stress diagnostic; run in release mode with --ignored --nocapture"]
fn offscreen_stroke_stress() -> Result<()> {
    unsafe {
        let (_window, renderer) = renderer()?;
        for density in [0, 25, 250] {
            let mut doc = document(density);
            // Each fixture is a new document, so invalidate the previous document's cache.
            *renderer.ink.borrow_mut() = None;
            let start = Instant::now();
            renderer.update_ink(&doc)?;
            let setup = start.elapsed();
            doc.begin_sample(Point::new(320., 240.), 1., None, 0);
            let start = Instant::now();
            for i in 1..=20_000 {
                doc.update_sample(
                    Point::new(
                        320. + (i as f32 * 0.07).sin() * 200.,
                        240. + (i as f32 * 0.09).sin() * 180.,
                    ),
                    None,
                    i * 4,
                );
                // Sample incremental updates without driving an OS pointer or enforcing real-time waits.
                if i % 80 == 0 {
                    renderer.update_ink(&doc)?;
                }
            }
            renderer.update_ink(&doc)?;
            assert_eq!(
                doc.active
                    .as_ref()
                    .unwrap()
                    .stroke
                    .as_ref()
                    .unwrap()
                    .nodes
                    .len(),
                20_001
            );
            assert_eq!(renderer.ink.borrow().as_ref().unwrap().completed, density);
            println!(
                "history={density}: cache setup={setup:?}, 20000 samples + cache updates={:?}",
                start.elapsed()
            );
        }
        Ok(())
    }
}
