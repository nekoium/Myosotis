# Myosotis — Windows prototype

A resident screenshot-and-scribble utility. Capture, mark, press Enter, paste.

## Run

Double-click **Start Myosotis.cmd**. The local Python environment is already installed on this machine.

- **Ctrl+Alt+S**: start capture on the monitor under the pointer.
- Left drag: screenshot region. Left click: full monitor.
- Right drag: sized white canvas. Right click: full-monitor white canvas.
- **1** pen, **2** arrow, **3** rectangle, **4** numbered mark.
- Draw with the mouse or stylus. Space also activates the tool at the pointer; hold it while moving to draw.
- Hover on a selection edge for **350 ms**, then drag to resize.
- **Z** undo, **Y** redo, **Enter** copy and close, **Escape** cancel.
- Closing the launcher leaves the app resident. Use the tray menu's **Quit** to exit.

Annotations retain their screen positions when resizing. Cropping clips annotations; expanding the selection reveals them again. The toolbar is not included in the clipboard image.

The app does not save or upload captures. Windows clipboard history/sync may retain copied images. Validation scripts can explicitly save synthetic test images under ignored `artifacts/`.

## Setup elsewhere

Requires Windows and Python 3.12+:

```powershell
python -m venv .venv
.venv\Scripts\python.exe -m pip install -r requirements.txt
.venv\Scripts\python.exe myosotis.py
```

## Checks

```powershell
.venv\Scripts\python.exe -m unittest -v
```

## Prototype limits

- PySide6 / Qt Widgets / QPainter; not yet the proposed C++ / Qt Quick production implementation.
- One active monitor per capture; regions cannot span monitors.
- Fixed pink ink and stroke width; stylus pressure/eraser not implemented.
- Fixed shortcut; a conflict is reported in the launcher.
- Simple toolbar painting is not yet exposed as accessible controls. Keyboard tool selection works.
- Capture timing in `artifacts/app.log` measures capture plus the request to show the overlay, **not** end-to-end input-to-photon latency.
- Real mixed-DPI multi-monitor, HDR, protected content, physical stylus hardware, and other operating systems are unvalidated.
- Qt licensing/distribution review and standalone packaging remain future work.

## Snapshots

`DESIGN.md` records the design. The first commit preserves the confirmed design before implementation.
