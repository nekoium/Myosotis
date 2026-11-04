# Myosotis 0.2.0 — More natural strokes

## A more natural mark

- Pen strokes now taper at the start and end. Mouse drawing adds mild speed-based width variation; valid stylus pressure controls width instead.
- Captured positions stay unchanged, without artificial path smoothing or a style change under load.
- Refined coral annotations and periwinkle controls bring the interface and copied marks together.
- Numbered badges center their visible numerals and widen to fit multiple digits.
- Updated flower artwork gives Myosotis a clearer identity in the notification area.

<!-- Manually upload 01-before-after.png here. -->

## Small controls, clear feedback

The compact toolbar keeps the same familiar pen, arrow, rectangle, numbered mark, Copy, and Cancel actions. Dark and Light themes share the new accent color, and contextual hints keep shortcuts close at hand.

Double-click or press Ctrl to copy; Enter and the Copy button remain available. These completion controls are unchanged.

<!-- Manually upload 02-components.png here. -->

## More ink, steady drawing performance

Completed annotations and the stable portion of the active stroke are cached, reducing repeated rendering work as a drawing grows.

A fresh comparison against 0.1.2 used five alternating runs per build, 120 timed observations per density, a 1400 × 850 physical-pixel canvas at 200% scaling, and a two-second 240 Hz curve replay at each density:

| Completed long strokes | 0.1.2 p95 paint time | 0.2.0 p95 paint time |
| --- | ---: | ---: |
| 0 | 10.75 ms | 10.67 ms |
| 25 | 13.78 ms | 10.78 ms |
| 100 | 49.02 ms | 10.81 ms |
| 250 | 132.97 ms | 10.90 ms |

These are single-machine forced-paint wall times, not input-to-display latency. All 9,200 checked reference positions in the 0.2.0 curve runs were within three pixels of ink. This coverage check does not prove exact sample retention or unlimited capacity.

The packaged executable also completed a 60-second, 240 Hz replay over 25 historical strokes. All 14,380 checked reference positions were within three pixels of ink; maximum injection lateness was 0.62 ms.

<!-- Manually upload 03-stress-test.png here. -->

## Download and update

Windows 11 x64 portable build. Quit the older Myosotis from its notification-area menu, extract `Myosotis-0.2.0-windows-x64.zip`, and run `Myosotis/myosotis.exe`. Otherwise, the older resident app will continue handling captures.

Scoop users: quit Myosotis, then run `scoop update myosotis` after this release and its manifest are published.

If you use Start at login, disable it before moving or deleting the previous portable copy, then enable it from the new executable location.

## Known limitations

Small antialias seams and preview/export pixel differences remain. Pressure was checked using synthetic Windows pen input; real stylus hardware remains unqualified. Display-presentation latency, 1000 Hz input, long-duration resource tracing, and broad DPI/multi-monitor coverage are not qualified.

History restoration and export still scale with drawing size. A 250-stroke copy check took 108.3 ms externally; long-stroke catch-up also remains proportional work. The proposed performance gate is not a claim that every workload meets a fixed latency budget.

The executable is unsigned. macOS and Linux are not included. Erasers and screen-reader access to individual toolbar controls are not implemented.
