# Input Schema Implementation Plan

**Goal:** Build the approved native shortcut reference/editor with icon-only Restore defaults.

**Architecture:** A shared binding model handles validation, labels and TOML persistence. A separate Win32 window reuses Direct2D toolbar icons and communicates binding changes to the resident application.

**Tech Stack:** Rust, existing windows and toml crates; no additional dependencies.

**Spec:** ../specs/2026-11-05-input-schema-design.md

## Global constraints

- Preserve user changes in CONTRIBUTING.md, src/windows/capture.rs, delegations/, and existing design artifacts.
- Keep the capture/drawing paths responsive and the desktop available during validation.
- Preserve theme, startup, gestures, Space drawing, and Enter completion.

## Review focus

- Conflicting/reserved bindings must never silently shadow an action.
- Failed writes and global registrations must preserve working bindings.
- Reset restores only shortcuts, preserving unrelated settings.
- Focus loss and held keys must not commit accidental changes.
- Layout must retain column alignment and legibility with DPI scaling.

## Task 1: Binding model and persistence

Files: src/input.rs (new); src/settings.rs; tests in these modules.

- [x] Add Action and Bindings with default keys, validation, labels, action lookup, Capture modifier chord and TOML load/save.
- [x] Test round-trip, invalid/duplicate/reserved keys, defaults, reset and unrelated/malformed settings preservation.
- [x] Reuse atomic settings writing for theme and bindings.

## Task 2: Native window and integration

Files: src/windows/schema.rs (new); src/windows/render.rs; src/windows/mod.rs; completion_tests.rs fixture.

- [x] Reuse toolbar icon drawing in a compact themed schema renderer, with fixed six-column layout and Reset/Close title controls.
- [x] Implement native window lifecycle, hover/focus descriptions, Tab navigation, key recording, cancellation, inline errors, and Reset.
- [x] Wire tray entry, startup load, safe global hotkey replacement, overlay action resolution and dynamic shortcut hints.
- [x] Add focused unit/offscreen checks for layout, binding changes and rendering without opening visible test windows.

## Task 3: Documentation and validation

Files: DESIGN.md; docs/INPUT.md; README.md.

- [x] Record approved behavior and configuration, update static shortcut language where needed.
- [x] Run formatting, Clippy and locked tests. Build a release for user review.
- [x] Inspect final diff and save a meaningful Rim state. Report native integration checks that still need a separately available desktop.

## Results

- `cargo fmt --check` and `git diff --check`: passed.
- `cargo clippy --all-targets -- -D warnings`: passed.
- `cargo test --locked`: 28 passed, one optional stress test ignored.
- `cargo test --locked --release offscreen_stroke_stress -- --ignored --nocapture`: passed. This checks memory-only document/cache work, not OS input or presentation latency.
- Schema rendering tested in both themes at 100%, 125%, 150%, and 200% scale. Native images in docs/design/input-schema/native-Dark.png and native-Light.png came from memory bitmaps, not desktop captures.
- Release build passed with `--target-dir target/input-schema`; executable is target/input-schema/release/myosotis.exe. The ordinary release output was locked, so the resident application was left untouched.
- Real tray interaction, hotkey registration contention/rollback, keyboard delivery and IME, foreground restoration, moving across differently scaled monitors, and visible presentation remain unqualified. No desktop automation or clipboard changes were performed. The custom schema does not yet provide UI Automation children.
- GitHub issue #10 was read once the connection recovered; it requests customization and a combined input reference/editor. No external issue changes were made.
