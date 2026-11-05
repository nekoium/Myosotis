use super::*;

#[test]
fn overlay_detaches_ime_without_changing_other_windows() {
    std::thread::spawn(|| unsafe {
        let create = || {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("STATIC"),
                PCWSTR::null(),
                WS_POPUP,
                0,
                0,
                1,
                1,
                None,
                None,
                GetModuleHandleW(None).unwrap(),
                None,
            )
            .unwrap()
        };
        let context = |hwnd| {
            let context = ImmGetContext(hwnd);
            if !context.is_invalid() {
                let _ = ImmReleaseContext(hwnd, context);
            }
            context
        };
        let previous = create();
        let overlay = create();
        let previous_context = context(previous);
        let layout = GetKeyboardLayout(0);
        let open = ImmGetOpenStatus(previous_context);
        disable_ime(overlay).unwrap();
        let overlay_context = context(overlay);
        let preserved_context = context(previous);
        let preserved_open = ImmGetOpenStatus(previous_context);
        let preserved_layout = GetKeyboardLayout(0);
        DestroyWindow(overlay).unwrap();
        DestroyWindow(previous).unwrap();
        assert!(
            overlay_context.is_invalid(),
            "IME must not intercept overlay keys"
        );
        assert_eq!(preserved_context, previous_context);
        assert_eq!(preserved_open, open);
        assert_eq!(preserved_layout, layout);
    })
    .join()
    .unwrap();
}
