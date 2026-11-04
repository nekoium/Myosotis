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
