# Input schema window

Related: issue #10. The user selected A1 after three visual rounds and requested implementation, then added an icon-only Reset beside Close.

## Intent and selected approach

Provide a compact native reference and editor for keyboard shortcuts, opened only through the tray menu's **Input schema** entry. Use the selected 344-DIP A1 arrangement: charcoal or light surface matching the active theme, custom title bar, Capture icon at left and shortcut at right, six toolbar icons with keycaps underneath, then Undo/Redo/Clear in the same first three columns. Reuse the exact existing toolbar geometry and periwinkle Copy treatment. No animation, shadow, new runtime, or configurable drawing appearance.

Alternatives explored were a named shortcut list, visual gesture guide, floating legend, and paired tiles. The user selected boxed shortcuts and familiar toolbar states, compact spacing, shared column centers, and right-aligned Capture. Mouse gesture instructions and secondary shortcuts are omitted from the window.

## Editing

Keycaps are editable without hover or focus descriptions. Clicking or keyboard activation listens for a replacement; Escape cancels, focus loss cancels, repeats do not commit. Tab moves among editable keys, Reset, and Close. Icons have no action-name tooltips. Show a short inline explanation for conflicting or unsupported keys. Existing Space drawing and Enter completion remain available and reserved. Overlay bindings use single keys; Capture supports a modifier chord. Save immediately to the existing settings file, preserving unrelated preferences and malformed files. Failed saves keep the old shortcut active. Update toolbar hints, tray text, and global registration along with bindings.

An icon-only Restore defaults control sits beside Close, with the hover/focus description “Reset all shortcuts”. It is disabled when defaults are active. Restore only input bindings; theme and startup are independent. Register the replacement Capture shortcut before releasing the current one, so rejection leaves the current shortcut usable. If a saved Capture shortcut is unavailable at startup, try the default and report the fallback.

## Window behavior

One normal, fixed-size window, independently draggable via its title area, DPI-scaled and placed within the current monitor work area. No always-on-top behavior. Closing hides it without quitting the resident app. Cancel listening before capture; hide the schema before screen capture so it does not appear in the result. Keep preview/export and drawing paths unchanged.

## Verification and limits

Test binding validation, resolution, serialization, reset, unrelated-setting preservation, failed saves, layout alignment/hit testing, and offscreen rendering. Run formatting, Clippy, and locked tests. Do not run real desktop input, focus, clipboard, or tray smoke checks without explicit desktop availability. Native presentation, registered-hotkey contention, screen reader behavior, and real multi-DPI/focus integration require separate qualification.
