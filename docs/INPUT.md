# Completion and canvas input

Agreed interaction specification for issue #2 and the related canvas controls.
This supplements DESIGN.md; its pre-existing local edits were intentionally left untouched.

## Completion

- Enter or Ctrl key-down copies the current selection and closes the overlay immediately. Repeated key-down messages do not repeat completion. Ctrl is a standalone completion key, not a modifier in the capture interface.
- Two left-clicks or two Space taps within 250 ms and less than 6 logical pixels of Manhattan displacement complete the capture. The interval is measured from first press to second press. Time and distance recognition are app-owned, independent of Windows double-click settings.
- Mouse and Space are separate recognition streams: a mouse click followed by Space is not a double-click.
- Before selection, the first click immediately selects the monitor under the pointer. A qualifying second click copies that monitor screenshot and closes. The tools may appear briefly between clicks.
- After selection, completion works inside and outside the selection. Toolbar actions and resize gestures do not participate. Border clicks remain eligible until the 350 ms resize-hover delay has elapsed.
- Drawing feedback is immediate. A click's annotation stays visible but uncommitted while a second click is possible. On completion, discard only that first click's annotation before exporting. Keep prior annotations and redo history intact. A brief first mark is acceptable.
- Movement crossing the drag threshold disqualifies the first click from completion. Long holds also expire. No waiting is added to drawing or selection feedback.
- Failed clipboard copying leaves the document open for retry, without the completion gesture's temporary mark. Completion never automatically pastes or sends.

## Shared pointer input

Space uses the same selection, toolbar, drawing, and resizing path as left-click. Holding Space while moving the pointer is equivalent to dragging. Auto-repeat is ignored. Simultaneous input from another source does not end the current gesture.

## Canvas controls

- Before selection, right-click selects a monitor-sized white canvas; right-drag selects a sized white canvas.
- After selection, right-click toggles white and captured-screenshot backgrounds, preserving annotations, selection, and tool. This is not an alpha-transparency mode.
- R clears annotations while preserving selection, background, and tool. Z undoes a reset as one action; Y redoes it. Numbered marks restart at 1 after reset.

## Validation

Unit tests cover recognition bounds, source separation, temporary-mark rollback, numbering, and reset history. `tools/Smoke-Test.ps1` checks native selection/completion, outside and border clicks, toolbar exclusions, Space input, clipboard pixels and failure recovery, background toggling, reset, and existing capture behavior. It takes focus and pointer control and replaces clipboard content; run it only in an available interactive desktop after quitting a resident Myosotis.

Automated smoke checks do not establish subjective drawing latency or accessibility across different users and hardware. The 250 ms threshold is a chosen product value, not a measured population average.
