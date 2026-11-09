use super::*;

#[test]
fn clipboard_image_validates_dimensions_and_normalizes_alpha() {
    assert!(Image::new(0, 1, &[]).is_err());
    assert!(Image::new(i32::MAX, i32::MAX, &[]).is_err());
    assert!(Image::new(2, 2, &[0; 4]).is_err());
    let image = Image::new(1, 2, &[1, 2, 3, 0, 4, 5, 6, 128]).unwrap();
    assert_eq!(image.pixels, [1, 2, 3, 255, 4, 5, 6, 255]);
    let dib = image.dib();
    assert_eq!(i32::from_le_bytes(dib[8..12].try_into().unwrap()), 2);
    assert_eq!(
        &dib[size_of::<BITMAPINFOHEADER>()..],
        &[4, 5, 6, 255, 1, 2, 3, 255]
    );
}

#[test]
fn clipboard_png_worker_round_trips_owned_pixels() {
    let image = Arc::new(
        Image::new(
            2,
            2,
            &[0, 0, 255, 0, 0, 255, 0, 1, 255, 0, 0, 128, 199, 70, 86, 255],
        )
        .unwrap(),
    );
    let expected = image.pixels.clone();
    let mut pending = Pending {
        owner: HWND::default(),
        format: 0,
        image,
        job: None,
    };
    pending.start();
    let png = pending.bytes().unwrap();
    assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok().unwrap();
        let result: Result<()> = (|| {
            let factory: IWICImagingFactory =
                CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER)?;
            let stream = factory.CreateStream()?;
            stream.InitializeFromMemory(&png)?;
            let decoder = factory.CreateDecoderFromStream(
                &stream,
                std::ptr::null(),
                WICDecodeMetadataCacheOnLoad,
            )?;
            let frame = decoder.GetFrame(0)?;
            let converter = factory.CreateFormatConverter()?;
            converter.Initialize(
                &frame,
                &GUID_WICPixelFormat32bppBGRA,
                WICBitmapDitherTypeNone,
                None,
                0.,
                WICBitmapPaletteTypeCustom,
            )?;
            let mut pixels = vec![0; expected.len()];
            converter.CopyPixels(std::ptr::null(), 8, &mut pixels)?;
            assert_eq!(pixels, expected);
            Ok(())
        })();
        CoUninitialize();
        result.unwrap();
    }
}

#[test]
fn clipboard_png_fallback_uses_the_existing_ui_com_apartment() {
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok().unwrap();
    }
    let result = encode(&Image::new(1, 1, &[1, 2, 3, 0]).unwrap());
    unsafe {
        CoUninitialize();
    }
    assert_eq!(&result.unwrap()[..8], b"\x89PNG\r\n\x1a\n");
}

#[test]
fn clipboard_worker_error_is_reported_without_panic() {
    let mut pending = Pending {
        owner: HWND::default(),
        format: 0,
        image: Arc::new(Image::new(1, 1, &[0; 4]).unwrap()),
        job: Some(std::thread::spawn(|| {
            Err(Error::new(E_FAIL, "test encoder failure"))
        })),
    };
    assert!(pending.bytes().is_err());
}

#[test]
fn clipboard_replacement_can_drop_an_in_flight_job() {
    let (send, receive) = std::sync::mpsc::channel();
    let pending = Pending {
        owner: HWND::default(),
        format: 0,
        image: Arc::new(Image::new(1, 1, &[0; 4]).unwrap()),
        job: Some(std::thread::spawn(move || {
            receive.recv().unwrap();
            Ok(vec![1])
        })),
    };
    drop(pending);
    // Dropping ownership detaches the worker, without waiting or publishing a completion.
    send.send(()).unwrap();
}
