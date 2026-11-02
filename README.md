# Myosotis

### Talk is cheap, show me the pic.

Screenshot, scribble, and paste into your coding agent.

Explaining a misplaced button can take longer than fixing it. Press **Ctrl+Alt+S**, select the area, draw what you mean, and hit **Enter**. Your image is on the clipboard, ready to paste into a chat or issue that accepts images.

Need to sketch an idea instead? Right-click for a blank canvas, or right-drag to choose its size.

**Windows prototype · Runs locally · No account required**

![Actual Myosotis clipboard output: a pink pen stroke and a numbered annotation on a white canvas.](assets/readme/clipboard-demo.png)

*Actual clipboard output from the prototype, created with the pen and numbered-mark tools. The toolbar is excluded from the exported image.*

## Made for visual feedback

- **Capture where you work.** A global shortcut opens a frozen view of the monitor under your pointer.
- **Mark what matters.** Use a pen, arrows, rectangles, or numbered marks without opening a separate editor.
- **Sketch without a screenshot.** Create a white canvas with a right-click or right-drag.
- **Keep your hands on the keyboard.** Number keys switch tools; Space uses the current tool at the pointer.
- **Copy and get back to work.** Enter copies the result and dismisses the overlay. Escape cancels without replacing your clipboard.

Myosotis stays in the system tray between captures. It does not save or upload your screenshots. Windows clipboard history or sync may retain images according to your settings.

## Try it on Windows

Requires **Python 3.12+** and Git. This is a source-based prototype; there is no standalone executable release yet.

Run in PowerShell:

```powershell
git clone https://github.com/nekoium/Myosotis.git
cd Myosotis
python -m venv .venv
.venv\Scripts\python.exe -m pip install -r requirements.txt
.venv\Scripts\python.exe myosotis.py
```

Once set up, double-click **Start Myosotis.cmd** to launch again.

Press **Ctrl+Alt+S**, drag a region, draw, press **Enter**, then paste into an application that accepts images. You can also start a capture from the launcher or tray menu. Closing the launcher leaves Myosotis running; choose **Quit** from the tray menu to exit.

## Controls

### Choose your surface

| Input | Result |
| --- | --- |
| Left drag | Capture a region |
| Left click | Capture the monitor under the pointer |
| Right drag | Create a sized white canvas |
| Right click | Create a white canvas filling the monitor |

### Annotate and finish

| Input | Action |
| --- | --- |
| `1` / `2` / `3` / `4` | Pen / arrow / rectangle / numbered mark |
| Mouse or stylus | Draw with the selected tool |
| `Space` | Use the tool at the pointer; hold and move to draw |
| Hover on an edge for 350 ms, then drag | Resize the selection |
| `Z` / `Y` | Undo / redo |
| `Enter` | Copy the image and close |
| `Esc` | Cancel and close |

Annotations stay in place when you resize. Shrinking clips them from the output; expanding reveals them again.

## Compatibility and limitations

Currently available for Windows. macOS and Linux are planned.

- Captures are limited to one monitor at a time.
- Ink is pink with a fixed stroke width.
- The shortcut is fixed to **Ctrl+Alt+S**. If another app has registered it, use the launcher or tray menu.
- Stylus pressure and eraser tools are not supported.
- Screen-reader support is limited; tools can be selected with the keyboard.
- HDR and mixed-DPI multi-monitor setups may have capture issues. Protected content may not be capturable.

## Feedback

[Report a bug or suggest an improvement](https://github.com/nekoium/Myosotis/issues). For capture issues, include your Windows version, display scaling, and steps to reproduce. Check screenshots for private information before posting them.

## License

Myosotis is available under the [MIT License](LICENSE).
