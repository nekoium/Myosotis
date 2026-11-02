# Myosotis

### Talk is cheap, show me the pic.

Screenshot, scribble, and paste into your coding agent.

Explaining a misplaced button can take longer than fixing it. Press **Ctrl+Alt+S**, select the area, draw what you mean, and press **Enter**. Paste the image into any chat, document, or issue that accepts images.

Need to sketch an idea instead? Right-click for a white canvas, or right-drag to choose its size.

## Get started

The native MVP is available from source for **Windows 11 x64**. It runs as a portable executable using Windows graphics and image facilities; no additional application runtime is required. There is no published, signed download yet. macOS and Linux are planned.

To build it, install [Rust](https://rustup.rs/) with the MSVC toolchain and the Visual Studio Build Tools **Desktop development with C++** workload, including a Windows SDK. From the repository:

```powershell
cargo build --locked --release
.\target\release\myosotis.exe --capture
```

Myosotis stays in the notification area after you copy or cancel. Use **Ctrl+Alt+S** to capture again. Click its flower icon to capture, or right-click it to quit. If Windows hides the icon, look under the notification area's overflow arrow.

Launching the executable again opens the resident app. The shortcut is fixed for this MVP; if another app owns Ctrl+Alt+S, Myosotis reports the conflict.

## Capture and sketch

The screen freezes when you invoke Myosotis.

- **Left-drag:** select a screenshot region. **Left-click:** select the monitor under the pointer.
- **Right-drag:** create a white canvas of that size. **Right-click:** use the whole monitor as a white canvas.
- Draw with the mouse or ordinary pen input, or select a tool from the floating toolbar.
- Hover over an edge or corner for a moment, then drag to resize. Annotations keep their positions; the exported image clips them to the selection.
- Press **Enter** to copy and close, then paste where you work. Press **Escape** to cancel and keep your previous clipboard.

The toolbar and selection handles stay out of the copied image. If copying fails because another app holds the clipboard, your drawing stays open so you can press Enter to retry.

## Controls

| Input | Action |
| --- | --- |
| `Ctrl+Alt+S` | Start a capture or white canvas |
| `1` / `2` / `3` / `4` | Pen / arrow / rectangle / numbered mark |
| Left mouse drag | Draw with the selected tool |
| `Space` | Use the tool at the pointer; hold and move to draw |
| Hover on an edge, then drag | Resize the selection |
| `Z` / `Y` | Undo / redo |
| `Enter` | Copy the image and close |
| `Esc` | Cancel and close |

## Privacy and limitations

Capture and annotation happen locally. Myosotis does not upload images, save captures to disk, or connect to your agent. It puts the finished image on the Windows clipboard as a bitmap and PNG. Windows clipboard history, synchronization, and other clipboard-reading applications can access it according to your system settings.

This MVP uses a fixed coral drawing color and stroke width. Pressure sensitivity, erasers, editable shapes, custom shortcuts, and screen-reader access to individual drawn toolbar controls are not implemented. Keyboard shortcuts are available for every tool, copy, and cancellation.

HDR output, protected content, secure desktops, and mixed-DPI multi-monitor arrangements have not been fully validated. Changing the display arrangement during a capture cancels that capture. Keep a separate copy of anything important before closing an overlay; annotations are not saved between captures.

## Feedback

[Suggest an improvement](https://github.com/nekoium/Myosotis/issues). Check screenshots for private information before posting them.

## License

Myosotis is available under the [MIT License](LICENSE).
