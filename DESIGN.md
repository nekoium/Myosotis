# Myosotis

## Purpose

“Talk is cheap, show me the pic.”

Make visual feedback for coding agents faster to communicate: invoke from anywhere, capture or open a blank canvas, scribble, copy, and paste into the agent.

## Product requirements

- Blazing fast; performance must not be compromised for portability.
- Available through a global hotkey.
- Minimal, clean interface that does not interrupt the current workflow.
- Windows first; macOS and Linux support planned.
- Clipboard output; no direct agent integration required.

## Interaction design

### Before selection

- Left drag: select a screenshot region.
- Left click: select the monitor under the pointer.
- Right click: create a white blank canvas filling the monitor under the pointer.
- Right drag: create a white blank canvas sized to the dragged region.
- Freeze the screen immediately when the global hotkey is pressed.

### After selection

- Mouse or stylus: use the selected drawing tool.
- Number keys (`1`, `2`, `3`, etc.): select the corresponding tool.
- Hover over a selection border briefly: enable selection resizing.
- Space: use the selected tool at the pointer position.
  - Numbered-mark tool: place a new numbered mark.
  - Pen tool: hold Space while moving the mouse to draw.
- `Z`: undo.
- `Y`: redo.
- Enter: copy the resulting image to the clipboard and close the overlay.

### At any point

- Escape: cancel and close.
- Global hotkey: initiate the workflow.

## Prototype defaults — subject to user testing

- Export uses the current selection bounds, including resized blank canvases.
- `1` pen (default), `2` arrow, `3` rectangle, `4` numbered mark.
- Click-versus-drag threshold: 6 logical pixels of Manhattan movement.
- Border hover delay: 350 ms; a resize cursor appears, then left drag resizes.
- Annotations keep their screen positions when resizing; export clips them to selection bounds.
- Discrete Space actions ignore auto-repeat; pen strokes end on Space release.
- Attempt to restore the previous foreground window after completion or cancellation.

## Open interaction decisions

- Simultaneous mouse, keyboard, and stylus input behavior beyond ordinary single-input use.
- Stylus pressure and eraser support.
- Accessibility and final toolbar placement.

## Implementation architecture

Use a shared Rust core with native implementations for each platform. Share the interface design and interaction behavior, not a bundled cross-platform UI renderer. Implement Windows first.

### Shared core

- Tool state, strokes, arrows, rectangles, and numbered marks.
- Selection geometry, undo/redo, and the input-action state machine.
- Drawing commands, style values, and export bounds.
- Use explicit coordinate conversions; keep OS handles and platform-specific coordinates out of the document model.

### Native platform responsibilities

- Windows, overlays, focus restoration, and tray integration.
- Capture, permissions, global shortcuts, clipboard formats, and mouse/stylus input.
- Rendering, text, and image encoding using operating-system facilities.
- Windows: Rust with Win32 and Direct2D / DirectWrite through the `windows` crate.
- macOS: native AppKit / Core Graphics integration; exact language bindings to be validated.
- Linux: native window-system integration and desktop portals; packaging and desktop support baseline to be validated.

### Reasons

- Keep the application small, responsive, clean, genuinely usable, and non-intrusive.
- Avoid shipping Python, Qt, Chromium, or another large application runtime.
- Rust keeps ordinary application logic memory-safe; isolate and review necessary unsafe platform API boundaries.
- Share behavior and visual specifications while respecting platform conventions and permissions.
- Accept more platform-specific implementation work in exchange for a smaller distribution and direct OS integration.
- The earlier Python prototype was already responsive in use. Migration is motivated by distribution size and native integration, not a demonstrated Python latency failure.

### Release-size rule

- Target each architecture-specific platform release at **less than 10 MB downloaded and less than 20 MB installed** (decimal MB).
- Measure the complete application, including bundled libraries and resources, not only the executable.
- Do not download an additional application runtime during installation. System-provided facilities are allowed; required Linux system packages must be documented.
- Publish debug symbols separately; avoid bundled fonts and large assets unless essential.
- Use release optimization, link-time optimization, and symbol stripping where supported; check artifact size automatically when release builds exist.
- Prove the Windows capture → annotate → clipboard flow within the budget before a full implementation.
- Prefer separate macOS architecture releases; universal binaries duplicate machine code and may exceed the budget.
- Define the Linux distribution/dependency baseline before promising portable bundles. A broadly self-contained AppImage may exceed the budget.
- These are acceptance targets, not measured claims. No native release artifact has established feasibility yet.

### Runtime behavior

- Resident process; keep disk I/O and image encoding out of the drawing path.
- Capture pixels and editable annotations remain separate until clipboard export.
- Measure invocation latency, stroke latency, memory use, and clipboard completion on real hardware.

### Platform risks to validate early

- Mixed-DPI and multiple-monitor coordinate mapping.
- HDR and protected-content capture restrictions.
- macOS screen-recording permission and application activation behavior.
- Linux Wayland capture/global-shortcut portals and compositor-dependent behavior.
- Image clipboard compatibility, contention, and ownership lifetime.
- Dependency licensing, native API safety, signing, and distribution obligations.

## Scope

This document records the product design and native architecture decision. The earlier Python implementation is retained locally only as an interaction reference. The public repository currently contains documentation; native implementation, cross-platform integration, release size, and production performance remain to be validated.
