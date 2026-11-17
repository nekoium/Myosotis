# Canvas lock design

Approved directly by the user for issue #20: Shift toggles lock once per press. While locked, rapid mouse/pen/Space taps remain drawing; gesture completion and keyboard Copy/Cancel (including Enter/Escape) are disabled. Toolbar Copy and Cancel remain available. Drawing/history/resizing remain unchanged. Each capture starts unlocked; Shift stays reserved/fixed and no preference is saved.

A black vector padlock matching toolbar stroke style sits inside the selection's top-right corner, persists while locked, and fades in/out. Exclude it from export. Implementation uses existing 160/100 ms opacity tween and Windows animation preference, with no input delays or added I/O.

Approaches: blocking gestures alone would leave accidental keyboard exits possible and does not match the user's clarification; blocking every exit would prevent the approved toolbar actions. Gate gesture/keyboard entry points while leaving the common copy/recovery path and toolbar actions intact.

Keep pending first-click ink/history when toggling and never interrupt an active mouse/pen/Space stroke. The indicator is not a control and must not intercept drawing. Tiny selections clip the indicator. Native desktop checks require separate explicit availability.
