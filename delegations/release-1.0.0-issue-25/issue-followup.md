Implemented on branch `nekoium/release-1.0.0-issue-25` in commit https://github.com/nekoium/Myosotis/commit/00aec458e1eb9c40c57de74112250cb077ae4103.

- [x] Persist the preference while preserving unrelated settings.
- [x] Add **Save to local** immediately below **Start at login** and rename **Dark theme**.
- [x] Save exact owned export pixels with shared background PNG encoding, Windows Pictures/Screenshots destination, collision-safe filenames, and normal-shutdown draining.
- [x] Cover persistence, concurrent filename collisions, encoding/destination errors, cancel/no-selection behavior, worker lifetime and decoded screenshot/white-canvas PNG pixels.
- [x] Update interaction/privacy docs and validation notes.

Validation: formatting and Clippy pass; full serial unit/offscreen suite passes (48 tests, 2 intentionally ignored), and release offscreen_stroke_stress passes. One existing live-keyboard-sensitive schema test failed intermittently in a parallel run, then passed isolated and serial runs; no schema code was changed. Native tray/error dialog interaction, redirected Pictures folders, and actual paste behavior remain separate qualification; no desktop/clipboard automation was run. Awaiting integration and combined release checks before closing.
