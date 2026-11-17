# Manual acceptance — combined local candidate

All boxes remain pending user testing. The worker did not launch either executable or qualify real input, presentation, or clipboard behavior.

Candidate: [myosotis.exe](C:/Users/Aria/orca/workspaces/Myosotis/release-1.0.0-candidate/dist/Myosotis/myosotis.exe) · [folder](C:/Users/Aria/orca/workspaces/Myosotis/release-1.0.0-candidate/dist/Myosotis)

Package/file/product version remains **0.2.1**. This is a local pre-release review candidate for the proposed 1.0.0 work, not a published 1.0.0 release. For manual comparison, quit the existing resident normally before launching a chosen executable; compare baseline and candidate separately on the same hardware, canvas, scaling, and input device.

- [ ] Press Shift once to lock; holding does not toggle repeatedly. Double-clicks, pen double-taps where hardware is available, and repeated Space taps draw without completion. Enter and configured Copy/Cancel keys cannot exit while locked. Drawing, tool changes, undo/redo, clear, and resizing still work. Shift unlocks; each fresh capture starts unlocked.
- [ ] Check the black top-right padlock fades in/out without disturbing drawing, including reversal/reduced motion where practical. Only toolbar Copy/Cancel exit while locked. Copy failure keeps the image available with toolbar retry. The padlock is absent from pasted and saved output.
- [ ] Tray shows **Dark theme** and **Save to local** directly below **Start at login**. With fresh settings, saving is off. With saving off, complete a capture and confirm no new file; enable it and confirm it survives normal Quit/restart, then disable and confirm that persists too.
- [ ] With saving on, finish screenshot and white-canvas captures with annotations and resized bounds. Check PNGs in the Windows Pictures known folder's `Screenshots` directory against the pasted image: same completed pixels, bounds, colors, annotations, and opaque background. Check both toolbar Copy while locked and ordinary unlocked completion.
- [ ] Finish several captures quickly; timestamp/suffixed filenames remain distinct and existing images survive. Cancel captures (including toolbar Cancel while locked) and verify no new file. A failed initial clipboard publication should save nothing; successful retry should save once.
- [ ] Copy then use normal tray Quit promptly: the completed PNG finishes saving and the image remains pasteable after exit. Replacing the clipboard with other content must preserve that newer content while local saving can finish independently.
- [ ] Compare empty-canvas and sustained/many-stroke drawing against the preserved baseline; check completion responsiveness and actual paste into the user's target app. Record any native error dialog, focus, redirected-Pictures, or input issue separately.
- [ ] Quick Input schema sanity: tray open/close, recording/cancel, theme, and capture hiding still behave normally. Report any opening flash as an observation only: #23 was closed **not planned**, with no confirmed correction or root cause.

User confirmation of the new combined binary is required before merging master. WinGet remains blocked until the user confirms the requested issues/disposition; no merge or distribution action is authorized by this checklist.
