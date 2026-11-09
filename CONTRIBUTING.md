# Contributing

## Ideas and product decisions

GitHub Issues are the primary record for unresolved product ideas, problems, and proposed improvements. [DESIGN.md](DESIGN.md) records agreed product principles and interaction decisions.

Before planning feature or UX work, review [open issues](https://github.com/nekoium/Myosotis/issues) and search existing issues for related discussions. Add evidence or candidate approaches to an existing issue when the problem and desired outcome match. Create a separate issue when the problem can be resolved independently, and link related issues.

Title issues around a specific user problem or desired outcome. Describe:

- **Problem:** Current behavior, the affected workflow, and why it matters. Distinguish observations from assumptions.
- **Desired outcome:** What users should be able to accomplish or understand, without prescribing a control or implementation.
- **Scope and constraints:** What the issue covers, its boundaries, and relevant requirements, including responsiveness.
- **Possible approaches:** Candidate solutions and tradeoffs. These remain proposals until a decision is recorded.
- **Open questions:** Evidence or decisions needed before implementation.
- **Decision:** Initially pending. Once an approach is selected, summarize the rationale and link supporting discussion and implementation work.

Use a tracking issue when several related problems need a shared overview. Link each child issue and keep the tracker's checklist current. Split implementation tasks only when they are useful to deliver an agreed approach; link them to the problem issue.

During planning, start from the relevant tracking issue and its linked discussions. The current [experience refinement tracker](https://github.com/nekoium/Myosotis/issues/5) covers completion with the left hand on the keyboard and right hand on the mouse, interface clarity, and annotation aesthetics. Record decisions in the issue body so readers do not need to reconstruct them from comments. When an agreed interaction changes, update DESIGN.md and link the implementation to its issue. Close a problem issue when its desired outcome is met, or record why it will not be pursued.

## Build

Use Rust's `x86_64-pc-windows-msvc` toolchain with the Visual Studio C++ build tools and a Windows SDK. The resource compiler embeds the icon, version information, and manifest. The repository's Cargo configuration statically links the C runtime.

```powershell
cargo build --locked
cargo run -- --capture
```

The platform-independent document, history, and geometry live in `src/document.rs`. Win32 input, the resident window and tray, GDI capture, Direct2D/DirectWrite rendering, WIC PNG encoding, and clipboard transfer live in `src/windows/`. Image coordinates are physical pixels relative to the virtual desktop; global screen coordinates are converted at the Windows boundary. Drawing dimensions use the invocation monitor's DPI scale.

## Check changes

Routine validation must leave the user's cursor, focus, screen, clipboard, and resident Myosotis available. Select checks for the changed behavior, run them after a meaningful change, and do not repeat passing checks merely because a conversation turn ended. Documentation-only changes need no application tests.

These checks do not inject desktop input or open visible test windows:

```powershell
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --locked
```

The unit tests cover history branching, numbered marks, tool paths, selection bounds, and resize coordinates. Windows rendering tests use memory bitmaps and an invisible window. They check numeral pixels and incremental ink-cache behavior through growing strokes, abandoned active ink, undo/redo/reset, and memory-only export. They do not need the resident app to quit.

Clipboard encoding tests validate PNG pixel round-tripping, DIB row order, and worker errors in memory. The separate `native_clipboard_delayed_png_lifecycle` test uses hidden windows and does not move the cursor or take focus, but **does replace the system clipboard** with synthetic images and text. Run it only with explicit clipboard-test authorization; it is ignored during routine `cargo test`:

```powershell
cargo test --locked native_clipboard_delayed_png_lifecycle -- --ignored --nocapture --test-threads=1
```

It checks immediate bitmap access, an early PNG request waiting for encoding, clipboard contention, newer text replacing pending PNG, consecutive image copies, encoding failure, and PNG surviving owner-window destruction. These checks do not qualify actual application paste compatibility or visible completion latency.

For a rendering change, this optional stress diagnostic also leaves the desktop available:

```powershell
cargo test --locked --release offscreen_stroke_stress -- --ignored --nocapture
```

It exercises 0, 25, and 250 completed strokes plus 20,000-sample active strokes, without real-time replay waits. Its timings measure document updates and cache work, not OS input delivery or visible presentation latency. It is not a numeric performance gate or a replacement for real input qualification.

Desktop tests below require `-Interactive`; without it they stop before loading GUI helpers or touching clipboard contents. Run them only when the user has made the desktop available for the entire run, or in a separate interactive Windows environment. Initialization is not the only disruptive part. Cursor movement during the suite can invalidate it. Do not automatically retry interrupted runs or treat background execution on the same desktop as isolation.

Use full desktop suites for a final release candidate or a change to their native integration behavior. Do not run them for every cosmetic variant, ordinary rendering iteration, or chat turn. Offscreen checks do not qualify global hotkeys, OS mouse/pen delivery, foreground restoration, screen freezing, tray menus, or clipboard ownership; keep those limitations explicit.

For an authorized native integration check, quit any resident Myosotis first, build a release, then run:

```powershell
cargo build --locked --release
powershell -NoProfile -STA -ExecutionPolicy Bypass -File tools/Smoke-Test.ps1 -Exe target/release/myosotis.exe -Interactive
```

The smoke script takes over mouse position and focus, invokes the real global shortcut, and replaces the clipboard with test data. It requires a primary display at `(0,0)` of at least 1400 × 1000 physical pixels, an unlocked interactive desktop, and a free Ctrl+Alt+S shortcut. It creates and removes its own resident app and color fixture. It verifies pixels from bitmap and PNG clipboard formats, drawing tools, selection resizing, Space input, undo/redo, clipboard contention, cancellation, frozen capture, foreground restoration, and clean exit. Timing logs go to `.tmp/smoke-timings.log`. Foreground restoration is asynchronous across processes; the fixture pumps its own event loop while checking activation.

`tools/Test-Themes.ps1` checks the native tray theme marks, default behavior without creating settings, restart persistence, both live palettes, capture/history preservation, and unchanged white-canvas/coral clipboard pixels:

```powershell
powershell -NoProfile -STA -ExecutionPolicy Bypass -File tools/Test-Themes.ps1 -Exe target/release/myosotis.exe -Interactive
```

Like the smoke check, it requires an interactive desktop, takes focus and pointer control, and replaces clipboard contents. Quit the resident app first. Settings are isolated under a temporary project folder by overriding `LOCALAPPDATA` only for the test child process; the test removes that folder afterward.

`tools/Verify-Clipboard.ps1` decodes a CF_DIB currently on the clipboard into `.tmp/clipboard.png` for diagnosis. Do not publish captures containing private information. The `ExecutionPolicy Bypass` flag applies to this PowerShell process only; these commands do not change the machine's execution policy.

## Native development tools

The stroke replay probe and release image compositor are Rust binaries in `tools/native`, separate from the application build and portable package. Their PowerShell entry points build the required release binary with the checked-in lockfile. The probe still requires `-Interactive` and exclusive desktop availability.

Check this separate Cargo package when changing these tools:

```powershell
cargo fmt --manifest-path tools/native/Cargo.toml --check
cargo clippy --manifest-path tools/native/Cargo.toml --locked --all-targets -- -D warnings
cargo test --manifest-path tools/native/Cargo.toml --locked
```

See [the migration record](docs/RUST-TOOLS-MIGRATION.md) for output comparisons, measured timings, and remaining interactive qualification.

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
