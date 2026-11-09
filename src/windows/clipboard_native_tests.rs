use super::*;
use std::{
    sync::mpsc,
    time::{Duration, Instant},
};

unsafe extern "system" fn owner_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if let Some(result) = message(hwnd, msg, wparam) {
        return result;
    }
    if msg == WM_DESTROY {
        shutdown(hwnd);
    }
    DefWindowProcW(hwnd, msg, wparam, lparam)
}

struct Owner(HWND);
impl Owner {
    unsafe fn new() -> Self {
        let instance = ::windows::Win32::System::LibraryLoader::GetModuleHandleW(None).unwrap();
        let class = w!("Myosotis.Test.ClipboardOwner");
        let wc = WNDCLASSW {
            lpfnWndProc: Some(owner_proc),
            hInstance: instance.into(),
            lpszClassName: class,
            ..Default::default()
        };
        RegisterClassW(&wc);
        Self(
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                class,
                w!(""),
                WS_POPUP,
                0,
                0,
                1,
                1,
                None,
                None,
                instance,
                None,
            )
            .unwrap(),
        )
    }
}
impl Drop for Owner {
    fn drop(&mut self) {
        unsafe {
            let _ = DestroyWindow(self.0);
        }
    }
}

unsafe fn open_clipboard() -> Result<()> {
    // Clipboard listeners can briefly hold it after each publication.
    let started = Instant::now();
    loop {
        match OpenClipboard(None) {
            Ok(()) => return Ok(()),
            Err(error) if started.elapsed() >= Duration::from_secs(1) => return Err(error),
            Err(_) => std::thread::sleep(Duration::from_millis(2)),
        }
    }
}

unsafe fn read_bytes(format: u32) -> Result<Vec<u8>> {
    open_clipboard()?;
    let result = (|| {
        let handle = HGLOBAL(GetClipboardData(format)?.0);
        let size = GlobalSize(handle);
        let ptr = GlobalLock(handle);
        if ptr.is_null() {
            return Err(Error::from_win32());
        }
        let bytes = std::slice::from_raw_parts(ptr.cast::<u8>(), size).to_vec();
        let _ = GlobalUnlock(handle);
        Ok(bytes)
    })();
    let _ = CloseClipboard();
    result
}

unsafe fn consume<T: Send + 'static>(
    operation: impl FnOnce() -> Result<T> + Send + 'static,
) -> Result<T> {
    let (send, receive) = mpsc::channel();
    let worker = std::thread::spawn(move || {
        send.send(operation()).unwrap();
    });
    let started = Instant::now();
    let result = loop {
        if let Ok(result) = receive.try_recv() {
            break result;
        }
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "clipboard request deadlocked"
        );
        let mut msg = MSG::default();
        while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
            DispatchMessageW(&msg);
        }
        std::thread::sleep(Duration::from_millis(1));
    };
    worker.join().unwrap();
    result
}

fn image(value: u8) -> Image {
    Image::new(2, 2, &[value; 16]).unwrap()
}
unsafe fn place(owner: HWND, value: u8) {
    let publication = publish(owner, image(value)).unwrap();
    start_encoding();
    drop(publication);
}
fn slow_job(bytes: Vec<u8>) -> JoinHandle<Result<Vec<u8>>> {
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(100));
        Ok(bytes)
    })
}

#[test]
#[ignore = "changes system clipboard; run only with explicit clipboard-test authorization"]
fn native_clipboard_delayed_png_lifecycle() {
    unsafe {
        let owner = Owner::new();
        let png_format = RegisterClipboardFormatW(w!("PNG"));
        let png = encode(&image(10)).unwrap();

        // Publish bitmap and advertise PNG without starting encoding or waiting for it.
        drop(publish(owner.0, image(10)).unwrap());
        assert!(IsClipboardFormatAvailable(8).is_ok());
        assert!(IsClipboardFormatAvailable(png_format).is_ok());
        assert!(PENDING.with(|slot| slot.borrow().as_ref().unwrap().job.is_none()));
        assert_eq!(&read_bytes(8).unwrap()[40..44], &[10, 10, 10, 255]);
        PENDING.with(|slot| slot.borrow_mut().as_mut().unwrap().job = Some(slow_job(png.clone())));
        let started = Instant::now();
        let received = consume(move || read_bytes(png_format)).unwrap();
        assert!(started.elapsed() >= Duration::from_millis(70));
        assert_eq!(&received[..png.len()], &png);
        assert!(PENDING.with(|slot| slot.borrow().is_none()));
        println!("PASS immediate bitmap and PNG request waiting on in-flight encoding");

        // A successful PNG worker can also publish proactively; clipboard contention retries.
        place(owner.0, 20);
        let (locked_send, locked_receive) = mpsc::channel();
        let (release_send, release_receive) = mpsc::channel();
        let lock = std::thread::spawn(move || {
            open_clipboard().unwrap();
            locked_send.send(()).unwrap();
            release_receive.recv().unwrap();
            CloseClipboard().unwrap();
        });
        locked_receive.recv().unwrap();
        std::thread::sleep(Duration::from_millis(20));
        message(owner.0, WM_TIMER, WPARAM(PNG_TIMER));
        assert!(PENDING.with(|slot| slot.borrow().is_some()));
        release_send.send(()).unwrap();
        lock.join().unwrap();
        message(owner.0, WM_TIMER, WPARAM(PNG_TIMER));
        assert!(PENDING.with(|slot| slot.borrow().is_none()));
        assert_eq!(&read_bytes(png_format).unwrap()[..8], b"\x89PNG\r\n\x1a\n");
        println!("PASS proactive PNG completion and retry after clipboard contention");

        // Losing ownership while a worker is running must not overwrite a newer item.
        drop(publish(owner.0, image(30)).unwrap());
        PENDING.with(|slot| slot.borrow_mut().as_mut().unwrap().job = Some(slow_job(png.clone())));
        consume(|| {
            open_clipboard()?;
            let result = (|| {
                EmptyClipboard()?;
                ClipboardMemory::new(&[b'N', 0, b'E', 0, b'W', 0, 0, 0])?.transfer(13)
            })();
            let _ = CloseClipboard();
            result
        })
        .unwrap();
        assert!(PENDING.with(|slot| slot.borrow().is_none()));
        std::thread::sleep(Duration::from_millis(120));
        message(owner.0, WM_TIMER, WPARAM(PNG_TIMER));
        assert_eq!(
            &read_bytes(13).unwrap()[..8],
            &[b'N', 0, b'E', 0, b'W', 0, 0, 0]
        );
        assert!(IsClipboardFormatAvailable(png_format).is_err());
        println!("PASS clipboard replacement during background encoding preserves newer text");

        // A locked clipboard must leave the previous item and pending state unchanged.
        let (locked_send, locked_receive) = mpsc::channel();
        let (release_send, release_receive) = mpsc::channel();
        let lock = std::thread::spawn(move || {
            open_clipboard().unwrap();
            locked_send.send(()).unwrap();
            release_receive.recv().unwrap();
            CloseClipboard().unwrap();
        });
        locked_receive.recv().unwrap();
        assert!(publish(owner.0, image(99)).is_err());
        release_send.send(()).unwrap();
        lock.join().unwrap();
        assert_eq!(
            &read_bytes(13).unwrap()[..8],
            &[b'N', 0, b'E', 0, b'W', 0, 0, 0]
        );
        println!("PASS initial publication failure preserves previous clipboard data");

        // A new image from the same owner cancels the old pending result.
        drop(publish(owner.0, image(40)).unwrap());
        PENDING.with(|slot| slot.borrow_mut().as_mut().unwrap().job = Some(slow_job(png.clone())));
        place(owner.0, 50);
        let expected = encode(&image(50)).unwrap();
        let received = consume(move || read_bytes(png_format)).unwrap();
        assert_eq!(&received[..expected.len()], &expected);
        println!("PASS consecutive copies use the latest image");

        // Failure of optional PNG leaves the immediate bitmap intact.
        drop(publish(owner.0, image(60)).unwrap());
        PENDING.with(|slot| {
            slot.borrow_mut().as_mut().unwrap().job = Some(std::thread::spawn(|| {
                Err(Error::new(E_FAIL, "injected encoder error"))
            }))
        });
        assert!(consume(move || read_bytes(png_format)).is_err());
        assert_eq!(&read_bytes(8).unwrap()[40..44], &[60, 60, 60, 255]);
        println!("PASS encoding failure retains usable bitmap");

        // Native window destruction must materialize delayed data for pasting after exit.
        drop(publish(owner.0, image(70)).unwrap());
        let expected = encode(&image(70)).unwrap();
        PENDING.with(|slot| {
            slot.borrow_mut().as_mut().unwrap().job = Some(slow_job(expected.clone()))
        });
        drop(owner);
        let received = read_bytes(png_format).unwrap();
        assert_eq!(&received[..expected.len()], &expected);
        assert!(PENDING.with(|slot| slot.borrow().is_none()));
        println!("PASS delayed PNG survives owner-window destruction");
    }
}
