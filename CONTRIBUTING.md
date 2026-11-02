# Contributing

## Build

Use Rust's `x86_64-pc-windows-msvc` toolchain with the Visual Studio C++ build tools and a Windows SDK. The resource compiler embeds the icon, version information, and manifest. The repository's Cargo configuration statically links the C runtime.

```powershell
cargo build --locked
cargo run -- --capture
```

The platform-independent document, history, and geometry live in `src/document.rs`. Win32 input, the resident window and tray, GDI capture, Direct2D/DirectWrite rendering, WIC PNG encoding, and clipboard transfer live in `src/windows/`. Image coordinates are physical pixels relative to the virtual desktop; global screen coordinates are converted at the Windows boundary. Drawing dimensions use the invocation monitor's DPI scale.

## Check changes

```powershell
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --locked
```

The unit tests cover history branching, numbered marks, tool paths, selection bounds, and resize coordinates.

For a native integration check, quit any resident Myosotis first, build a release, then run:

```powershell
cargo build --locked --release
powershell -NoProfile -STA -ExecutionPolicy Bypass -File tools/Smoke-Test.ps1 -Exe target/release/myosotis.exe
```

The smoke script takes over mouse position and focus, invokes the real global shortcut, and replaces the clipboard with test data. It requires a primary display at `(0,0)` of at least 1400 × 1000 physical pixels, an unlocked interactive desktop, and a free Ctrl+Alt+S shortcut. It creates and removes its own resident app and color fixture. It verifies pixels from bitmap and PNG clipboard formats, drawing tools, selection resizing, Space input, undo/redo, clipboard contention, cancellation, frozen capture, foreground restoration, and clean exit. Timing logs go to `.tmp/smoke-timings.log`. Foreground restoration is asynchronous across processes; the fixture pumps its own event loop while checking activation.

`tools/Verify-Clipboard.ps1` decodes a CF_DIB currently on the clipboard into `.tmp/clipboard.png` for diagnosis. Do not publish captures containing private information. The `ExecutionPolicy Bypass` flag applies to this PowerShell process only; these commands do not change the machine's execution policy.

## Package

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools/Package.ps1
```

This builds a Windows x64 portable folder and ZIP under `dist/`, includes user documentation and license notices, writes a SHA-256 checksum, and rejects packages at or above 10,000,000 downloaded bytes or 20,000,000 installed bytes. The produced folder contains no bundled runtime or external DLLs. Build symbols and build-only dependencies stay outside the package. Quit the copy running from `dist/` before rebuilding that package.

To regenerate the checked-in Windows icon:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools/New-Icon.ps1
```

See [DESIGN.md](DESIGN.md) for behavior and architecture, and [docs/VALIDATION.md](docs/VALIDATION.md) for the current validation limits.
