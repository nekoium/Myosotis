# Input schema transitions — approved design

Issue: https://github.com/nekoium/Myosotis/issues/18

Status: user approved this design and authorized implementation in the issue worker session after selecting 160 ms opening, 100 ms closing, and 60 ms color (200 ms opening was tried and rejected as slow). Native candidate is implemented and the user authorized finalizing #18. The user still observes an opening flash; its unresolved cause and further correction are tracked separately in [issue #23](https://github.com/nekoium/Myosotis/issues/23). GitHub decision and medium scope are recorded in [issue #18](https://github.com/nekoium/Myosotis/issues/18#issuecomment-6719452900).

## Intent

Remove abrupt entrance, dismissal, and control-color changes without making animation draw attention. Apply only to Input schema. Keep the screenshot and annotation interface animation-free.

## Selected timing

- Fade the schema in over 160 ms and out over 100 ms, without translation, scaling, or bounce.
- Blend control background and foreground colors over 60 ms, including hover entry/exit, focus, and recording-state changes. Text and shortcut actions update immediately.
- Use the preview's ease-out curve, `cubic-bezier(.2,0,0,1)`, for all transitions. Opening and closing have separate durations; the closing preview has been updated for review.

## Interaction constraints

Implementation details included in the approved design:

- Keep input responsive during opening. Beginning dismissal immediately cancels recording and stops accepting schema input; the remaining fade is visual only.
- Close, valid Escape dismissal, and focus loss use the closing fade, preserving #14's recording and dismissal rules. Never steal focus back during dismissal.
- Starting capture bypasses any fade, hides the schema synchronously, and cancels animation work before desktop capture. No animation delays capture or leaves schema pixels in the screenshot.
- Reopening during dismissal reverses from current opacity; an already-visible schema does not replay entrance. Interrupted color transitions continue from their displayed color.
- User-requested follow-up: drag from all non-control areas, retaining button/keycap click targets. Background dragging cancels recording. Tray left-click toggles visibility, preserving button-down intent across focus-loss dismissal.
- Hold opacity while showing and painting the first frame; start the entrance clock after the visible HWND render completes. Show/foreground calls must release the WINDOW borrow so synchronous paint/focus messages can run.
- Respect the Windows animation preference: disabled animations make all transitions immediate. Stop scheduling animation work at rest and when hidden.
- Preserve #14 behavior and #15 appearance, window dimensions, layout, shortcut validation, and persistence.

## Approaches

The browser study compared instant changes and 80/60, 120/90, and 160/120 ms entrance/color timings. User initially selected 160 ms entrance, tried 200 ms, and returned to 160 ms because 200 ms felt slow. They also reported flashing during opening and requested background dragging and tray visibility toggling. Closing remains 100 ms and color remains 60 ms. Fade-only avoids spatial movement. Longer color blending would make rapid hovering feel less direct; instant entrance retains the reported abruptness.

## Delivery and acceptance

Treat production work as medium scope because opacity and timed rendering touch native window lifecycle and can affect responsiveness. Preserve runnable baseline and candidate builds for user comparison. Validate interruption/reopening, immediate capture hiding, reduced motion, both themes, and existing offscreen checks. Live native presentation and focus checks require separate user testing or explicit desktop availability. Browser timing is a design reference, not native performance evidence.

The user accepted the current interactions and authorized finalization after trying the candidate, while explicitly deferring the remaining opening flash to #23. This acceptance does not establish that flashing is fixed or that all hardware/focus/presentation scenarios are qualified. Native implementation uses the existing Windows APIs and no additional runtime dependencies.
