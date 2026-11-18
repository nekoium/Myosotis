# Myosotis 1.0.0 — Capture it before you even realize

Press **Ctrl+Alt+S**, select a screenshot or white canvas, draw what you mean, and copy it into your chat, document, or issue.

Since 0.2.1, Myosotis adds protection for longer drawing sessions, optional local PNG saving, and background PNG encoding after the capture closes.

## Keep drawing until you're ready

Press **Shift** to lock the canvas. Double-clicks, pen double-taps, and repeated Space taps keep drawing instead of finishing. Copy and Cancel keyboard shortcuts are disabled while locked; use the toolbar buttons to finish, or press Shift to unlock. Each capture starts unlocked, and the padlock stays out of your finished image.

![Canvas lock keeps repeated taps drawing; illustrated comparison with 0.2.1](https://raw.githubusercontent.com/nekoium/Myosotis/v1.0.0/docs/releases/1.0.0/01-canvas-lock.png)

## Keep a copy

Enable **Save to local** in the tray menu to save each successfully copied capture as a PNG in **Pictures\Screenshots**. The file includes your selected area and annotations. Saving runs in the background, never overwrites existing files, and is off by default. Canceled captures create no file.

![Optional Save to local sends the completed image to both clipboard and PNG; illustrated workflow](https://raw.githubusercontent.com/nekoium/Myosotis/v1.0.0/docs/releases/1.0.0/02-save-to-local.png)

## Capture before you even realize

PNG compression now runs in the background after the overlay closes. Apps accepting bitmap clipboard data can paste immediately; apps requesting PNG before encoding finishes still wait for it.

![Completion order before and after background PNG encoding](https://raw.githubusercontent.com/nekoium/Myosotis/v1.0.0/docs/releases/1.0.0/03-background-png.png)

## Small refinements, shown up close

- A simpler tray menu with a direct **Dark theme** toggle.
- Input schema dismisses on Escape outside shortcut recording or when you switch windows, stays out of the taskbar and Alt+Tab, and can be dragged from non-control areas.
- Subtle Input schema opening, closing, and control-color transitions respect Windows' animation preference.
- Refined header alignment, improved Windows icon sizing, and optical alignment for single-digit numbered marks.

![Input schema reference and interaction refinements](https://raw.githubusercontent.com/nekoium/Myosotis/v1.0.0/docs/releases/1.0.0/04-refinements.png)

## Download and update

**Windows 11 x64 · Portable · No additional runtime required**

Quit the older Myosotis from its notification-area menu, extract `Myosotis-1.0.0-windows-x64.zip`, and run `Myosotis/myosotis.exe`.

If you use **Start at login**, turn it off before moving or deleting the previous portable copy, then enable it from the new executable's location.
