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

## Open interaction decisions

- Blank-canvas export bounds after resizing.
- Initial tool set, number-key assignments, and default tool.
- Click-versus-drag movement threshold.
- Border hover delay, resize activation, and visual feedback.
- How existing annotations behave when the selection is resized.
- Space key-repeat behavior for discrete marks; handling simultaneous mouse, keyboard, and stylus input.
- Stylus pressure and eraser support.
- Focus restoration after completion or cancellation.

## Proposed implementation direction — not yet confirmed

- Windows implementation first; preserve a path to macOS and Linux through Qt.
- Prototype may use PySide6 to validate interaction without a C++ toolchain; this does not establish C++ performance.
- C++ with Qt 6 / Qt Quick remains the proposed production stack.
- Small platform-specific adapters for capture, global hotkeys, clipboard export, and window behavior.
- Resident process; keep disk I/O and image encoding out of the drawing path.
- Capture pixels and editable annotations remain separate until clipboard export.
- Measure invocation latency, stroke latency, memory use, and clipboard completion on real hardware.

### Platform risks to validate early

- Mixed-DPI and multiple-monitor coordinate mapping.
- HDR and protected-content capture restrictions.
- macOS screen-recording permission and application activation behavior.
- Linux Wayland capture/global-shortcut portals and compositor-dependent behavior.
- Image clipboard compatibility, contention, and ownership lifetime.
- Qt licensing and distribution obligations.

## Scope

This document records the proposed product design. No application implementation or final technology selection has been made.
