use super::*;

fn click(input: Input, age_ms: u64) -> Click {
    Click {
        point: Point::new(20., 20.),
        since: Instant::now() - Duration::from_millis(age_ms),
        input,
    }
}
fn app() -> App {
    App {
        hwnd: HWND::default(),
        previous: HWND::default(),
        doc: Document::default(),
        renderer: None,
        theme: Theme::Dark,
        bindings: Bindings::default(),
        input_hints: InputHints::new(&Bindings::default()),
        hotkey_id: HOTKEY,
        pixels: vec![],
        origin: POINT::default(),
        full: Rect::default(),
        scale: 1.,
        visible: false,
        pointer: Point::default(),
        gesture: None,
        click_start: None,
        pending_click: None,
        space: false,
        pen_id: None,
        pen_clock: 0,
        hover: None,
        hint: String::new(),
        tray: NOTIFYICONDATAW::default(),
        tray_icon: TrayIcon::default(),
        taskbar_created: 0,
        schema_tray_close: None,
    }
}

#[test]
fn failed_tray_refresh_preserves_the_existing_owned_icon() {
    unsafe {
        let mut app = app();
        app.tray_icon = TrayIcon::small().unwrap();
        app.tray.hIcon = app.tray_icon.0;
        let original = app.tray_icon.0;
        // No tray entry is registered by this fixture; NIM_MODIFY must fail without adding one.
        app.refresh_tray_icon(NIM_MODIFY);
        assert_eq!(app.tray.hIcon, original);
        assert_eq!(app.tray_icon.0, original);
        let mut info = ICONINFO::default();
        GetIconInfo(original, &mut info).unwrap();
        let _ = DeleteObject(info.hbmColor);
        let _ = DeleteObject(info.hbmMask);
        drop(app);
        assert!(GetIconInfo(original, &mut ICONINFO::default()).is_err());
    }
}

#[test]
fn tray_toggle_keeps_close_intent_after_focus_loss_finishes_hiding_schema() {
    let mut app = app();
    app.schema_tray_close = Some(true);
    unsafe {
        app.message(TRAY, WPARAM(0), LPARAM(WM_LBUTTONUP as isize));
    }
    assert_eq!(app.schema_tray_close, None);
    assert!(
        !schema::visible(),
        "button-up must not reopen a schema hidden after button-down"
    );
}

#[test]
fn configured_tool_keys_and_guidance_use_the_same_bindings() {
    let mut app = app();
    app.bindings.set(Action::Arrow, b'W' as u16).unwrap();
    app.bindings.set(Action::Cancel, b'C' as u16).unwrap();
    app.input_hints = InputHints::new(&app.bindings);
    unsafe {
        app.key(b'W' as usize, false);
        assert_eq!(app.doc.tool, Tool::Arrow);
        app.key(0x31, false);
        assert_eq!(app.doc.tool, Tool::Pen);
        app.key(0x32, false);
        assert_eq!(
            app.doc.tool,
            Tool::Pen,
            "old binding must no longer select Arrow"
        );
    }
    assert!(app.input_hints.toolbar[1].contains("W"));
    assert!(app.input_hints.selection.ends_with("C cancel"));
    assert!(app.input_hints.workflow.ends_with("C cancel"));
}

#[test]
fn recognition_uses_app_time_distance_and_separate_input_sources() {
    let first = click(Input::Mouse, 100);
    assert!(first.matches(Point::new(22., 21.), Input::Mouse, 1.));
    assert!(!first.matches(first.point, Input::Space, 1.));
    assert!(!first.matches(Point::new(26., 20.), Input::Mouse, 1.));
    assert!(first.matches(Point::new(26., 20.), Input::Mouse, 2.));
    assert!(!click(Input::Mouse, 251).matches(first.point, Input::Mouse, 1.));
    assert!(click(Input::Space, 100).matches(first.point, Input::Space, 1.));
}

#[test]
fn completion_discards_only_the_visible_uncommitted_click_and_preserves_redo() {
    let mut app = app();
    app.doc.select(
        Rect::between(Point::default(), Point::new(100., 100.)),
        false,
    );
    app.doc.begin(Point::new(10., 10.), 1.);
    app.doc.finish();
    app.doc.undo();
    app.doc.tool = Tool::Number;
    app.doc.begin(Point::new(20., 20.), 1.);
    app.pending_click = Some(click(Input::Mouse, 0));
    assert!(app.doc.active.is_some());
    app.discard_click();
    assert!(app.doc.active.is_none());
    assert!(app.doc.annotations.is_empty());
    app.doc.redo();
    assert_eq!(app.doc.annotations.len(), 1);
    assert_eq!(app.doc.annotations[0].tool, Tool::Pen);
}

#[test]
fn ordinary_click_commits_once_and_keeps_numbering_sequential() {
    let mut app = app();
    app.doc.select(
        Rect::between(Point::default(), Point::new(100., 100.)),
        false,
    );
    app.doc.tool = Tool::Number;
    for _ in 0..2 {
        app.doc.begin(Point::new(20., 20.), 1.);
        app.pending_click = Some(click(Input::Space, 0));
        app.flush_click();
        app.flush_click();
    }
    assert_eq!(app.doc.annotations.len(), 2);
    assert_eq!(app.doc.annotations[1].number, 2);
}

#[test]
fn selection_and_outside_click_completion_preserve_document() {
    let mut app = app();
    let selection = Rect::between(Point::default(), Point::new(100., 100.));
    app.doc.select(selection, false);
    app.pending_click = Some(click(Input::Mouse, 0));
    app.discard_click();
    assert_eq!(app.doc.selection, Some(selection));
    assert!(!app.doc.blank);
    assert!(app.doc.annotations.is_empty());
    assert!(app.doc.active.is_none());
}
