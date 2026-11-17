Confirmed with the user for 1.0.0:

- Add a persistent, checkable **Save to local** tray item directly below **Start at login**, off by default.
- While enabled, every successfully completed capture is also saved as a PNG under the Windows **Pictures\Screenshots** folder. Use the Windows Pictures location so redirected Pictures folders are respected.
- Save the same selected bounds, annotations, and screenshot/white background as clipboard output; exclude toolbar and selection handles.
- Name files using local time, e.g. `Screenshot 2026-11-15 215736.png`, and avoid overwriting when timestamps collide.
- Preserve the existing clipboard workflow and fast completion; encode/write in the background without a Save As dialog. Canceled captures produce no file.
- Create the Screenshots folder when needed. Report save failures while preserving the clipboard image.
- Rename **Dark Theme** to **Dark theme**.

Implementation checklist:

- [ ] Persist the preference while preserving unrelated settings.
- [ ] Add the tray checkbox and update theme capitalization.
- [ ] Save owned export pixels in a background worker with collision-safe filenames and error/shutdown handling.
- [ ] Cover preference, file collisions, failures, cancellation and correct image pixels in focused unit/offscreen checks.
- [ ] Update interaction/privacy docs and record actual validation limits.

Implementation is underway on isolated branch `nekoium/release-1.0.0-issue-25`; the implementation commit will be recorded in the local result for release integration.
