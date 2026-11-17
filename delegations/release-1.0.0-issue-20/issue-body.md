## Problem

During longer drawing sessions, accidental double-taps can copy and close the capture before the user intends to finish. Original report: “To avoid mis-double click and end while drawing.” Related completion gesture discussion: #2.

## Desired outcome

Users can protect a drawing session from accidental completion while keeping drawing immediate and retaining deliberate toolbar completion/cancellation.

## Confirmed decision

[Direct user approval and interaction details](https://github.com/nekoium/Myosotis/issues/20#issuecomment-6952698085):

- Shift toggles canvas lock once per press; it is fixed/reserved and ignores auto-repeat.
- Locked mouse double-clicks, pen double-taps, and repeated Space taps remain ordinary drawing input.
- While locked, Enter and configured Copy/Cancel shortcuts are disabled; only the quick toolbar Copy/Cancel buttons can intentionally finish/cancel.
- Drawing, tool switching, history, clearing, Space drawing, and resizing remain available.
- A black vector lock icon appears inside the captured field's top-right corner, fades in, stays visible while locked, and fades out on unlock. It is an overlay indicator excluded from export.
- New captures start unlocked; lock state is not saved.

Implementation choices: reuse existing 160/100 ms fade and Windows animation preference; retain shared clipboard recovery and avoid drawing delays or I/O.

## Tracking

- [x] Confirm interaction with user.
- [x] Implement input gates, lock indicator, hints and reset.
- [x] Preserve ordinary drawing/history and toolbar completion paths.
- [x] Add focused input and memory rendering checks at common DPI.
- [x] Pass formatting, Clippy, routine tests (47 passed, 2 intentionally ignored), release build and explicitly run offscreen stroke stress.
- [x] Update DESIGN.md and README.md.
- [ ] Confirm visible fade, real pen/mouse double-tapping, toolbar Copy/clipboard recovery, and baseline/candidate interaction on an explicitly available desktop.

Implementation is on branch `nekoium/release-1.0.0-issue-20`; see the linked implementation comment for commit and evidence. Native qualification remains separate; no desktop automation was performed.
