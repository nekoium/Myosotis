# Save to local — approved design and implementation plan

User approval: direct chat confirmation followed by “First updated the confirmed change to issue by adding a reply, and do it.” Decision recorded at https://github.com/nekoium/Myosotis/issues/25#issuecomment-6952374179.

## Design

The tray gains persistent **Save to local**, off by default, immediately below **Start at login**. Every successful clipboard completion while enabled also saves its exact flattened selection as PNG to the Windows Pictures known folder's `Screenshots` subfolder. Local-time names use `Screenshot 2026-11-15 215736.png`; collisions append ` (2)`, ` (3)`, etc. Canceled captures and failed initial clipboard publication do not save. No dialog or new input action is added. Clipboard completion stays fast; encoding and disk I/O run on workers after hiding the overlay. Save errors report that the clipboard image remains available. Quit waits for outstanding saves. Theme label becomes **Dark theme**.

Approaches considered: manual Save last image versus automatic saving of completed captures. User selected automatic saving through a persistent tray checkbox and specified folder/naming. Reuse the existing WIC encoder and export snapshot; share its PNG result across clipboard/local workers, preserving clipboard ownership independent from saving.

## Plan

- [x] Add `SaveToLocal` load/save preference in `src/settings.rs`, preserving unrelated TOML and prior settings on failed writes; test default, round-trip, invalid values and preservation.
- [x] Add `src/windows/local_save.rs` with known Pictures location, local timestamp, exclusive collision-safe file creation, cleanup on write failure, background jobs, polling error reporting and shutdown joins. Test collision, directory/write/encoding failures, unchanged input pixels, jobs surviving snapshot/clipboard release and shutdown draining.
- [x] Let `clipboard::Publication` retain an `Arc<Image>` snapshot and cache PNG encoding in `Image`; keep clipboard message/lifetime behavior unchanged. Reuse existing memory-only PNG decoding checks.
- [x] Wire `src/windows/mod.rs` tray preference/startup and successful-copy snapshot dispatch, background completion polling and shutdown; no save on cancel or copy failure. Rename theme label and update affected desktop script expectations without running the script.
- [x] Update DESIGN.md, README.md and validation notes for actual disk writes and remaining native tray/redirected-folder qualification.
- [x] Run cargo fmt, clippy, locked unit/offscreen checks; review diff; commit and record result/integration notes.

## Review focus

Timestamp collisions must never overwrite. File or folder failures must preserve clipboard output. In-flight saves must survive clipboard replacement and ordinary shutdown. Disabled/canceled operations must not create image files. Malformed settings must be preserved. Worker errors must not panic or publish stale clipboard content.
