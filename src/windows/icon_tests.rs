use super::*;

#[test]
fn embedded_icon_has_native_frames_for_common_windows_sizes() {
    let ico = include_bytes!("../../assets/myosotis.ico");
    assert_eq!(&ico[..4], &[0, 0, 1, 0]);
    let count = u16::from_le_bytes([ico[4], ico[5]]) as usize;
    let sizes = [16, 20, 24, 32, 40, 48, 64, 96, 128, 256];
    assert_eq!(count, sizes.len());
    for (entry, size) in ico[6..6 + count * 16].as_chunks::<16>().0.iter().zip(sizes) {
        let dimension = |byte: u8| if byte == 0 { 256 } else { byte as u32 };
        assert_eq!(dimension(entry[0]), size);
        assert_eq!(dimension(entry[1]), size);
        let length = u32::from_le_bytes(entry[8..12].try_into().unwrap()) as usize;
        let offset = u32::from_le_bytes(entry[12..16].try_into().unwrap()) as usize;
        let png = &ico[offset..offset + length];
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
        assert_eq!(u32::from_be_bytes(png[16..20].try_into().unwrap()), size);
        assert_eq!(u32::from_be_bytes(png[20..24].try_into().unwrap()), size);
    }
}

#[test]
fn tray_icons_load_at_requested_sizes_and_release_owned_handles() {
    unsafe {
        let shared = LoadIconW(
            GetModuleHandleW(None).unwrap(),
            PCWSTR(std::ptr::without_provenance(1)),
        )
        .unwrap();
        for size in [16, 20, 24, 32, 40, 48, 64, 96, 128, 256] {
            let icon = TrayIcon::load(size, size).unwrap();
            assert_ne!(icon.0, shared, "tray must not own the shared class icon");
            let mut info = ICONINFO::default();
            GetIconInfo(icon.0, &mut info).unwrap();
            let mut bitmap = BITMAP::default();
            assert_ne!(
                GetObjectW(
                    info.hbmColor,
                    size_of::<BITMAP>() as i32,
                    Some((&mut bitmap as *mut BITMAP).cast()),
                ),
                0
            );
            assert_eq!((bitmap.bmWidth, bitmap.bmHeight), (size, size));
            let _ = DeleteObject(info.hbmColor);
            let _ = DeleteObject(info.hbmMask);
            let handle = icon.0;
            drop(icon);
            assert!(
                GetIconInfo(handle, &mut ICONINFO::default()).is_err(),
                "owned icon must be destroyed on drop"
            );
        }
        // Dropping the tray icons must leave the shared window-class resource valid.
        let mut info = ICONINFO::default();
        GetIconInfo(shared, &mut info).unwrap();
        let _ = DeleteObject(info.hbmColor);
        let _ = DeleteObject(info.hbmMask);

        let small = TrayIcon::small().unwrap();
        GetIconInfo(small.0, &mut info).unwrap();
        let mut bitmap = BITMAP::default();
        assert_ne!(
            GetObjectW(
                info.hbmColor,
                size_of::<BITMAP>() as i32,
                Some((&mut bitmap as *mut BITMAP).cast()),
            ),
            0
        );
        let taskbar = FindWindowW(w!("Shell_TrayWnd"), PCWSTR::null()).unwrap_or_default();
        let dpi = GetDpiForWindow(taskbar);
        let dpi = if dpi == 0 { GetDpiForSystem() } else { dpi };
        assert_eq!(bitmap.bmWidth, GetSystemMetricsForDpi(SM_CXSMICON, dpi));
        assert_eq!(bitmap.bmHeight, GetSystemMetricsForDpi(SM_CYSMICON, dpi));
        let _ = DeleteObject(info.hbmColor);
        let _ = DeleteObject(info.hbmMask);
    }
}
