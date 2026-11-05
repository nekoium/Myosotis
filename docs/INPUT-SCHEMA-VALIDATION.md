# Input schema visual correction

Reference: docs/design/input-schema/selected-dark.png (688 × 588 pixels at 200% display scale), reaffirmed by the user on 2026-11-05. The Reset icon requested after that image remains beside Close.

## Proven defects and fixes

- The first native implementation was only 282 DIP high, compared with 294 DIP in the approved preview. Bottom keycaps had six pixels of breathing room at 200%. Restore 294 DIP and 21 DIP below the bottom keycap row.
- The secondary divider and controls were too low. Divider now sits at 192 DIP; secondary keycaps start at 248 DIP. Both rows retain the same column centers.
- Character-count width guesses made the Capture keycap much too wide and padded Ctrl/Esc inconsistently. DirectWrite text metrics now determine keycap bounds; the renderer and pointer hit testing use the same measurement method.
- The title used normal 11-DIP text. It now uses 12-DIP Segoe UI Semibold with reference-aligned placement. Close and Reset use compact 14-DIP icons rather than oversized 20-DIP glyphs.
- Capture had corner brackets rather than the approved dashed square. Undo/Redo curves and Clear's open circular arrow now follow the preview shapes.
- The native tooltip existed but had zero registered tools. The unthemed system common-controls library rejected the newest TOOLINFO structure size. Use the compatible structure size up to lpReserved, provide initial text, and verify registration succeeds. A regression test asserts exactly one native tool without showing a window.
- Mouse hints now follow the active theme. Keyboard focus hints draw inside the window because the native tooltip dismisses on keyboard delivery. This was confirmed visually after Tab navigation.
- Avoid reapplying the same window size/region during ordinary focus changes. Clearing an edit error and reselecting the existing binding now clears stale error feedback without queuing a write. Clicking empty space clears the focus indication.

## Verification

- Inspected the actual native windows with the computer-use skill and Orca, rather than only memory rendering. Screenshots: docs/design/input-schema/live-dark.png and live-light.png. Their idle layout is 688 × 588 at the machine's 200% scale.
- Opened Input schema through the resident tray menu; inspected dark/light, recording, error feedback, keyboard focus hints, and Reset. Changed Pen to Q in an isolated LOCALAPPDATA profile, verified the stored value was 81, then clicked Reset and verified it returned to 49. No user preference or clipboard changes were made.
- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test --locked`: passed, 29 tests passed and one optional stroke stress diagnostic ignored. The existing memory-only rendering checks cover both themes at 100%, 125%, 150%, and 200%. Added measured keycap width, row placement, bottom-padding, and keyboard-hint checks.
- Built both target/release/myosotis.exe and the previously linked target/input-schema/release/myosotis.exe. Restarted the corrected ordinary release using the user's original environment/preferences and opened Input schema for review.
- This pass does not qualify capture/clipboard output, all global shortcut conflicts, multi-monitor DPI transitions, or full screen-reader navigation. GUI checks were confined to Input schema and its tray options.

## Subsequent removal of messages

The user subsequently requested silent rejection and no descriptive text in Input schema. Removed the native tooltip control and its registration, keyboard description rendering, error text, and variable-height error area. Invalid keys leave recording active and bindings unchanged; failed registration or saves retain the previous bindings. No rejection outline or other error feedback replaces the text. The title, key labels, normal focus outlines, and recording indicator remain.

Formatting, Clippy, and locked tests pass: 28 passed, one optional diagnostic ignored. The removed tooltip registration test accounts for the reduced count. The hidden-window state test now checks silent invalid-key rejection. Release built to `target/input-schema`; no live desktop checks were run for this removal.

Rim candidate: schema-remove-all-messages, n9ea84053c568e30.

Rim candidate: input-schema-visual-fix, n2e2b502e02d1448. Parent: input-schema-native, naa8edcafdce4f5e.
