# Incremental stroke experiment results

Status: Natural selected and approved by the user for commit after speed/stress testing. Natural is enabled in standard builds; Clean remains available for diagnostic comparison with `--no-default-features`. No automatic style fallback, position smoothing, or prediction is present.

## Approval

The user chose Natural after comparing the profiles and confirmed that the speed/stress test passed. The earlier recommendation of Clean below records the evaluator's preference at the time, not the final product choice. Approval accepts the change with the documented visual and qualification limits; it does not turn the proposed performance gate into a fully implemented or passed gate.

Build the selected profile with `cargo build --locked --release`. Build Clean separately with `cargo build --locked --release --no-default-features --target-dir .tmp/stroke-clean-comparison` to preserve the standard build.

## Directions

| Direction | Mouse behavior | Stylus behavior | Result |
| --- | --- | --- | --- |
| Clean, primitive renderer | Short start/end taper, otherwise steady width | Valid pressure controls width | Preferred: simplest profile and lower history/export cost |
| Natural, primitive renderer | Same taper plus mild speed-based width response | Pressure takes precedence over speed simulation | Similar interactive performance, modest visible benefit, more variable width |
| Small winding-path batches | Same width rules; union at most 32 captured segments per path | Same pressure model | Rejected: 20,000-sample catch-up paint reached 146.7 ms; preview/export edge differences increased |

The original giant-path experiments remain failed references. They were not rebuilt or overwritten.

## Implementation

Captured positions remain unchanged. Samples are not filtered or spatially resampled into a dense stroke representation. Width is computed separately, using pressure when valid and mouse speed only in Natural. Straight segments split only at a few width-profile landmarks, so a two-event flick still has a full-width body. These derived vertices stay on the captured straight segment and do not pretend to recover missing motion.

A transparent Direct2D bitmap retains completed annotations and stable active ink. Frames restore the screenshot/blank background, composite stable ink, and redraw the short trailing taper. The immutable document history remains available for rebuilding the cache and uncached export. Selection clipping happens after caching, preserving annotation positions on resize. Undo/reset/redo invalidate the derived cache. Canceled or discarded active annotations cannot leave cached ink behind.

Profiles have fixed behavior across workloads and no F6 switching. Endpoint widths do not change at lift-off. There is no load-dependent quality or style change.

## Run discipline and artifacts

The final source snapshot is `.tmp/stroke-v2/final-source/`. It includes concurrent palette and numbered-badge work already present in the shared workspace. That work was preserved. The early candidate binaries used the preceding palette; final matched comparisons use only the frozen final snapshot. Comparison screenshots therefore show different colors between some diagnostic phases without implying a stroke-width change.

Native tests use a 1400 × 850 physical-pixel white canvas on the local 2560 × 1600 display at 200% scaling. Final density comparisons alternate build order across five runs and collect 24 timed observations per density per run after five warmups: 120 measured observations per build and density. These are forced-paint and external dispatch wall times, not display-presentation latency. The release 0.1.2 baseline executable remains unchanged.

External mouse movement disturbed early batches 6 and 7. Both are excluded from scored comparisons; the entire early series is superseded by the final series. The replay now rejects lost focus, hidden capture, or detected pointer displacement between injections. A focus/pointer check is not a complete proof that no external input occurred.

Raw CSVs, injection timestamps, and screenshots are in `.tmp/stroke-eval/results/final-*/`. `.tmp/stroke-v2/summary.json` records executable hashes, sizes, quantiles, and secondary diagnostic results. Regenerate it with `node tools/stroke-eval/SummarizeV2.cjs`.

## Measured performance

Final matched p95 forced-paint wall times, milliseconds (120 observations per cell):

| Historical long strokes | Baseline | Clean | Natural |
| --- | ---: | ---: | ---: |
| 0 | 10.80 | 10.76 | 10.76 |
| 5 | 10.82 | 10.91 | 10.76 |
| 25 | 13.83 | 10.86 | 10.88 |

Candidate external input-dispatch p95 stayed between 0.31 and 0.34 ms. Maximum combined dispatch-plus-paint time in these final matched runs was 11.45 ms for Clean and 11.28 ms for Natural, versus 15.07 ms for baseline. All 6,900 reference positions per build across final curve runs were within three pixels of ink. This mask-based diagnostic cannot establish exact sample retention or replace centerline instrumentation.

Single-run stress diagnostics, 24 observations per density:

| Historical long strokes | Baseline paint p95 | Clean | Natural |
| --- | ---: | ---: | ---: |
| 100 | 48.96 | 10.90 | 10.80 |
| 250 | 132.56 | 10.85 | 10.51 |

At 250 strokes, baseline missed 218 of 460 reference positions by more than three pixels; both candidates missed zero. Baseline stress controls predate final palette integration; baseline itself is unchanged.

Long-active-stroke diagnostics separate the first paint after untimed paint-suppressed setup from five subsequent paints. Small counts are observed maxima, not reliable production percentiles:

| Prepared samples | Baseline subsequent-paint maximum | Clean | Natural |
| --- | ---: | ---: | ---: |
| 640 | 10.96 ms | 10.88 ms | 11.11 ms |
| 5,000 | 32.62 ms | 11.01 ms | 10.84 ms |
| 20,000 | 115.62 ms | 10.95 ms | 10.99 ms |

The first catch-up paint at 20,000 samples was 123.86 ms for baseline, 37.13 ms for Clean, and 35.81 ms for Natural. The remaining proportional catch-up work is not eliminated by caching; these observations do not establish the proposed 33.3 ms p99 budget.

At 250 completed strokes, undo/redo/reset restoration paints reached 50.38 ms for Clean and 68.87 ms for Natural, versus 143.62 ms for baseline. Full synchronous copy dispatch took 76.80 ms for Clean, 100.49 ms for Natural, and 108.83 ms for baseline. The application logged export/clipboard completion at 58 ms for Clean and 81 ms for Natural; the external measurement includes additional window/dispatch work. Natural's external copy exceeds the proposed 100 ms stall boundary and remains a failure to investigate, even though baseline is slower. Baseline's result does not excuse it.

Each final executable is 571,392 bytes versus baseline's 544,256 bytes. Native smoke cycles ended with 269 handles and resident working sets of 77.8 MiB (Clean) and 71.9 MiB (Natural). These short runs do not establish leak freedom or GPU-memory bounds. The ink bitmap alone requires approximately four bytes per full-canvas pixel, in addition to existing capture/render resources.

## Functional and quality checks

- Unit checks cover exact captured corners/tiny movements, timestamp wrap, distinct absent/zero pressure, stationary pressure changes, stable prefix widths, short marks, two-event flicks, and pressure overriding mouse speed. Existing document/history/completion checks remain enabled.
- Native smoke checks cover mouse/Space completion, pending-click rollback, all tools, undo/redo/reset, resizing, screenshot/blank modes, bitmap and PNG clipboard output, clipboard contention, cancellation, repeated invocation, and foreground restoration.
- Synthetic Windows pen injection exercises actual pointer messages, pen history, pressure validity, and promoted-mouse suppression. Light/heavy/ramping pressure produces distinct visible widths. This is not qualification of real stylus hardware.
- Shapes include intentional right angles, loops/crossings, dots, two-event flicks, and Space curves. No giant chord collapse was observed in uncontaminated fixtures.
- The primitive renderer still shows small antialias seams where independently rendered pieces meet. Cached preview and uncached export are not pixel-identical. Final primitive shape tests found 5 ink-mask pixel differences for Clean and 9 for Natural, with none more than one pixel from corresponding ink. This remains a quality limitation, not a full matching-output pass.
- A 60-second, 240 Hz real-input replay on 25 historical strokes completed for baseline and both primitive directions. All 14,380 interior reference positions in each run were within three pixels of ink. Maximum injection lateness was under 0.7 ms. This checks final coverage, not input-to-photon latency or exact processed-sample counts.

## Remaining limitations

The full proposed performance gate is not implemented. Real stylus hardware, presentation latency, broad DPI/multi-monitor coverage, 1000 Hz input, long-duration resource/leak tracing, and rigorous centerline/sample accounting remain unqualified. Dense local input can leave many samples inside the trailing taper, and history/export still scale with document size. No claim of unlimited constant-time rendering is made.

Visual seams and exact preview/export agreement remain unresolved quality limitations accepted with this commit. The batched winding alternative is rejected because its worst observed catch-up stall violates the proposed 100 ms boundary. Natural also recorded a 100.49 ms synchronous external copy dispatch at 250 strokes, exceeding that proposed boundary. Clean has the better measured margin; the user nevertheless selected Natural and approved the speed/stress result. These observations remain recorded for follow-up.

## Try the experiments

Quit the resident app through its tray menu first. Then run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools/Try-StrokesV2.ps1 -Direction Clean
powershell -NoProfile -ExecutionPolicy Bypass -File tools/Try-StrokesV2.ps1 -Direction Natural
powershell -NoProfile -ExecutionPolicy Bypass -File tools/Try-StrokesV2.ps1 -Direction Baseline
```

Run one command at a time; finish/cancel and quit before changing builds. These commands launch separate frozen local executables. They do not change startup registration or the installed app.

## Workspace preservation

During evaluation, Rim repeatedly reported that no root existed for the current project context even though `.rim/` and historical objects remained. It was not reinitialized or switched. Pre-edit source and frozen final source snapshots were saved under `.tmp/stroke-v2/`. Existing Git changes and concurrent palette/badge edits were preserved. Before committing the approved profile, Rim worked again and preserved candidate `natural-stroke-approved` (`nf9b20f02306100d`). The palette/badge changes were committed separately. This stroke approval does not publish a release.
