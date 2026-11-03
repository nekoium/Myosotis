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
- Windows: Rust with Win32 and Direct2D / DirectWrite through the `windows` crate. The MVP captures SDR desktop pixels through GDI and encodes PNG through Windows Imaging Component. Clipboard output includes CF_DIB and PNG.
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

## Selected interface direction

The finalized Windows interface defaults to a compact dark rectangular toolbar with rounded corners, no perimeter outline, and no shadow. Six 40-DIP cells contain vector icons for pen, arrow, rectangle, numbered mark, Copy, and Cancel. Neutral fills identify the selected tool and hovered controls; blue emphasizes Copy. The existing guidance surface uses the same dark treatment, a shorter persistent hint, and contextual tool descriptions on hover. Keyboard shortcuts, completion gestures, and toolbar placement remain unchanged.

Annotation defaults use coral ink, rounded pen and arrow caps, finer arrows and rectangle outlines, and compact numbered markers. Overlay rendering and clipboard export share these defaults. No animations or background sampling are added.

### Appearance preference

For [issue #6](https://github.com/nekoium/Myosotis/issues/6), the native tray menu provides a **Theme** submenu with mutually exclusive **Dark** and **Light** radio choices. Dark is the first-launch default. Changing the choice immediately repaints toolbar and guidance surfaces, text, icons, and control states without resetting a capture or its history. Copy retains its blue fill and white icon. Captured pixels, white blank canvases, annotation styles, and clipboard export are independent of the interface theme. The native popup menu follows Windows styling.

The preference lives in `%LOCALAPPDATA%\Myosotis\settings.toml`, using the TOML library already present in the build dependency graph. Read once at startup; create the folder and settings file only when the user changes the theme. Missing, invalid, or unreadable settings fall back to Dark. Save through a temporary file and replace the previous file after writing succeeds. A failed save leaves the new theme active for the session and displays an understandable error. Existing unrelated TOML values survive a theme update; comments and formatting may be rewritten. Malformed TOML is preserved rather than overwritten.

There is no registry setting for the theme or automatic system-theme mode. Settings remain when a portable executable is deleted; users can reset preferences by closing Myosotis and deleting the settings folder. Storage outside the executable folder is an intentional choice so read-only app folders and executable moves work.

This direction was selected after comparing Material, light macOS-inspired, and dark rectangular candidates with and without an outline. See [the design comparison](docs/DESIGN-DIRECTIONS.md) for references, verification, and remaining accessibility and display checks. This is a native Windows design; it does not add macOS support.

### Start at login

For [issue #7](https://github.com/nekoium/Myosotis/issues/7), the native tray menu includes a checkable **Start at login** item. Startup is opt-in: launching Myosotis does not create or modify registration. Enabling writes a quoted, absolute path to the current executable as a `REG_SZ` value named `Myosotis` under `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`. No capture argument is included, so login starts only the resident tray app. Disabling deletes only that value, leaving other startup entries untouched. No administrator rights are required.

The menu rereads the registry whenever it opens, rather than persisting a second preference. Its check mark indicates registration exists, including an entry pointing to a previous executable location. If registration exists and its `StartupApproved\Run` record is recognized as disabled, the label becomes **Start at login (Disabled in Windows)** while remaining checked and selectable. Selecting it still removes registration. Without registration, the annotation is never shown, even if an old approval record remains. Approval is read-only and best-effort: absent, unreadable, malformed, or unknown records retain the ordinary label. This Windows registry format is undocumented; detection recognizes the known 12-byte binary record with a four-byte disabled state of `3`. Myosotis never changes Windows' approval state; users re-enable it in Task Manager if needed. Unreadable registration disables the item with a state-unavailable label and reports the error after the menu closes. Failed changes report an understandable error; reopening the menu always reads the actual state. Windows startup controls and policy may independently block execution. Portable users must disable startup before moving or deleting the executable, then enable it from the new location if needed. Theme reset does not affect startup.

## Experience refinement goals

Myosotis should make the capture → annotate → copy workflow immediately understandable, with minimal reading and configuration. These goals guide future work; they do not describe features already implemented.

- Users should be able to finish a capture while keeping the left hand in its keyboard working area and the right hand on the mouse, without reaching for Enter. Double-click completion is a candidate; single-click recognition and unintended drawing actions need discussion. Keyboard shortcuts remain available.
- Controls should communicate their purpose with minimal on-screen text. Use recognizable icons where they improve clarity; retain labels where they prevent ambiguity. Preserve discoverability and accessible names.
- Annotations should look clean, attractive, and coherent with the tool's visual design. Explore line-weight variation and color treatment for aesthetic quality while keeping the default workflow free of configuration.
- Interface quality should come from legibility, consistent spacing, alignment, and clear control states. Decorative shadows and animations are not a current priority.
- Blazing-fast invocation, drawing, and completion are non-negotiable. Refinements must preserve responsiveness and the existing release-size budget.

Unresolved problems, candidate approaches, and implementation discussion belong in GitHub Issues. Start with the [experience refinement tracker](https://github.com/nekoium/Myosotis/issues/5). Agreed interaction decisions belong in this document. See [CONTRIBUTING.md](CONTRIBUTING.md#ideas-and-product-decisions) for the issue workflow.

## Scope

This document records the product design and native architecture decision. The Windows MVP now implements the capture → annotate → clipboard workflow, resident tray integration, and the interaction defaults above. The document model is platform-independent Rust; Windows API calls are isolated in `src/windows/`. Release builds statically link the C runtime and embed an application icon, version information, and a per-monitor DPI awareness manifest. `tools/Package.ps1` builds the complete portable package and enforces the download and installed size budgets.

The earlier Python implementation remains locally as an interaction reference. Native macOS and Linux implementations, pressure/eraser support, full toolbar accessibility, HDR handling, and broad multi-monitor validation remain future work. Windows integration and performance evidence are recorded in `docs/VALIDATION.md`.
