use super::*;
use crate::{
    document::{Document, Point, Rect, Tool},
    settings::Theme,
};
use windows::{
    core::*,
    Win32::{Foundation::HWND, UI::WindowsAndMessaging::*},
};

struct Directory(PathBuf);
impl Directory {
    fn new(name: &str) -> Self {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(".tmp")
            .join(format!("local-save-{name}-{}", std::process::id()));
        assert!(!path.exists());
        Self(path)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        if self.0.exists() {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }
}
fn time() -> SYSTEMTIME {
    SYSTEMTIME {
        wYear: 2026,
        wMonth: 11,
        wDay: 15,
        wHour: 21,
        wMinute: 57,
        wSecond: 36,
        ..Default::default()
    }
}

#[test]
fn local_save_names_and_concurrent_collisions_never_overwrite() {
    let directory = Directory::new("collisions");
    let handles: Vec<_> = (0..6u8)
        .map(|value| {
            let path = directory.0.clone();
            std::thread::spawn(move || {
                let bytes = [value];
                save(&path, time(), || Ok(&bytes)).unwrap()
            })
        })
        .collect();
    let paths: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    assert_eq!(
        paths.iter().collect::<std::collections::HashSet<_>>().len(),
        6
    );
    for (value, path) in paths.iter().enumerate() {
        assert_eq!(fs::read(path).unwrap(), [value as u8]);
    }
    assert!(directory
        .0
        .join("Screenshot 2026-11-15 215736.png")
        .exists());
    for index in 2..=6 {
        assert!(directory
            .0
            .join(format!("Screenshot 2026-11-15 215736 ({index}).png"))
            .exists());
    }
}

#[test]
fn local_save_encoding_and_destination_errors_leave_existing_work_intact() {
    let directory = Directory::new("errors");
    assert!(save(&directory.0, time(), || Err(io::Error::other(
        "encoding failure"
    )))
    .is_err());
    assert!(
        !directory.0.exists(),
        "encoding failure must not create a file or folder"
    );
    fs::create_dir_all(&directory.0).unwrap();
    let blocker = directory.0.join("not-a-folder");
    fs::write(&blocker, b"previous data").unwrap();
    assert!(save(&blocker, time(), || Ok(b"png")).is_err());
    assert_eq!(fs::read(blocker).unwrap(), b"previous data");
    assert_eq!(fs::read_dir(&directory.0).unwrap().count(), 1);
}

#[test]
fn local_save_jobs_survive_source_release_and_shutdown_collects_success_and_errors() {
    let directory = Directory::new("shutdown");
    let image = Arc::new(Image::new(1, 1, &[1, 2, 3, 255]).unwrap());
    let worker_image = image.clone();
    let path = directory.0.clone();
    let job = std::thread::spawn(move || {
        save(&path, time(), || {
            worker_image
                .png()
                .map_err(|e| io::Error::other(e.to_string()))
        })
        .map_err(|e| e.to_string())
    });
    drop(image);
    let mut saves = Saves {
        jobs: vec![job, std::thread::spawn(|| Err("injected failure".into()))],
    };
    assert_eq!(saves.collect(true), ["injected failure"]);
    assert!(saves.is_empty());
    let bytes = fs::read(directory.0.join("Screenshot 2026-11-15 215736.png")).unwrap();
    assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
}

struct HiddenWindow(HWND);
impl Drop for HiddenWindow {
    fn drop(&mut self) {
        unsafe {
            let _ = DestroyWindow(self.0);
        }
    }
}

#[test]
fn offscreen_local_png_uses_selected_bounds_annotations_and_both_backgrounds(
) -> windows::core::Result<()> {
    let directory = Directory::new("export");
    unsafe {
        let window = HiddenWindow(CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("STATIC"),
            w!("Local save offscreen"),
            WS_POPUP,
            0,
            0,
            160,
            120,
            None,
            None,
            None,
            None,
        )?);
        let pixels = [21, 43, 65, 255].repeat(160 * 120);
        let renderer =
            super::super::render::Renderer::new(window.0, 160, 120, &pixels, 1., Theme::Dark)?;
        let full = Rect::between(Point::default(), Point::new(160., 120.));
        for blank in [false, true] {
            let mut doc = Document::default();
            doc.select(
                Rect::between(Point::new(10., 20.), Point::new(110., 100.)),
                blank,
            );
            doc.tool = Tool::Rectangle;
            doc.begin(Point::new(30., 40.), 1.);
            doc.update(Point::new(90., 80.));
            doc.finish();
            let surface = renderer.export(&doc, &pixels, full)?;
            assert_eq!((surface.width, surface.height), (100, 80));
            let image = Image::new(surface.width, surface.height, surface.pixels())?;
            let path = save(&directory.0, time(), || {
                image.png().map_err(|e| io::Error::other(e.to_string()))
            })
            .unwrap();
            let bytes = fs::read(path).unwrap();
            assert_eq!(bytes, image.png()?);
            let decoded = decode(&bytes)?;
            assert_eq!(decoded.len(), 100 * 80 * 4);
            assert_eq!(
                &decoded[..4],
                if blank {
                    &[255, 255, 255, 255]
                } else {
                    &[21, 43, 65, 255]
                }
            );
            assert!(
                decoded.as_chunks::<4>().0.contains(&[86, 70, 199, 255]),
                "saved file lost annotations"
            );
            assert!(decoded.as_chunks::<4>().0.iter().all(|p| p[3] == 255));
        }
    }
    Ok(())
}

unsafe fn decode(bytes: &[u8]) -> windows::core::Result<Vec<u8>> {
    use windows::Win32::{Graphics::Imaging::*, System::Com::*};
    CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;
    let result = (|| {
        let factory: IWICImagingFactory =
            CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER)?;
        let stream = factory.CreateStream()?;
        stream.InitializeFromMemory(bytes)?;
        let decoder = factory.CreateDecoderFromStream(
            &stream,
            std::ptr::null(),
            WICDecodeMetadataCacheOnLoad,
        )?;
        let frame = decoder.GetFrame(0)?;
        let mut width = 0;
        let mut height = 0;
        frame.GetSize(&mut width, &mut height)?;
        assert_eq!((width, height), (100, 80));
        let converter = factory.CreateFormatConverter()?;
        converter.Initialize(
            &frame,
            &GUID_WICPixelFormat32bppBGRA,
            WICBitmapDitherTypeNone,
            None,
            0.,
            WICBitmapPaletteTypeCustom,
        )?;
        let mut pixels = vec![0; width as usize * height as usize * 4];
        converter.CopyPixels(std::ptr::null(), width * 4, &mut pixels)?;
        Ok(pixels)
    })();
    CoUninitialize();
    result
}
