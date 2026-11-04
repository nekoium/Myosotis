# Proposed stroke performance evaluation

Status: **evaluation proposal for discussion**, informed by [the reproduction findings](STROKE-PERFORMANCE-FINDINGS.md). This is not yet an implemented release gate, and its budgets have not been approved. The policy of rejecting failing effects and preserving consistent styling, without automatic fallback under load, is agreed. Baseline remains the default reference. Only Clean and Natural remain in scope; Brush is retired.

## Intent

Prevent the new failures: slow input handling, increasingly expensive active strokes, redraw cost that grows with accumulated annotations, and lost trajectory detail during fast drawing. Blazing-fast responsiveness is the first principle. No appearance benefit can compensate for failing the performance gate.

Performance must be achieved without changing what the user drew. Artificial position smoothing, prediction, and curve fitting are excluded. Clean and Natural can differ in width behavior, including pressure and optional speed-based width response. Width filtering must not change position samples or add path lag.

## Approaches considered

1. **Deterministic offscreen benchmarks only.** Repeatable and useful for isolating geometry costs, but miss the native input queue, coalescing, visible redraw, and presentation. Insufficient as the gate.
2. **Visible replay and manual judgment only.** Reproduce the real failure, but timing noise and screenshot-only scores make small regressions hard to locate and easy to hide. Insufficient alone.
3. **Two-layer evaluation with functional boundaries.** Use deterministic cost/fidelity checks plus real OS input and visible presentation checks. Retain independent history/export/geometry checks so a faster but incorrect renderer cannot pass. This is the recommended approach and the basis of this proposal, not a selected renderer architecture.

## Reference and run discipline

- Freeze the baseline source commit and record executable hashes, compiler flags, and any diagnostic instrumentation. Keep an unmodified release run as a control.
- Compare release builds under the same machine, display, DPI, canvas size, power state, and input trace. Log hardware, refresh rate, driver version, and scheduler/injection timing.
- Interleave candidate and baseline runs in alternating order. Use warm-up and at least five independent runs. Collect at least 120 measured paint/input observations per required scenario. Keep p50, p95, p99, maximum stall, and failure counts per scenario; do not average away bad cases.
- Validate the driver before scoring a run. Reject timing-distorted, occluded, unfocused, truncated, or timed-out runs. A candidate-induced timeout is a failure, not an excluded fast result.
- Prepare identical document content before each timed scenario. Exclude setup from interactive timing, but separately measure sustained natural drawing so fast setup cannot hide input or accumulated-content problems.
- Compare the original fixed-width behavior and both effect directions. Keep direction-specific width assertions separate from position fidelity.

## Layer 1: deterministic work and fidelity

Replay captured input samples, including timestamps and pressure, through the document and rendering stages. Measure:

- Time spent accepting samples, deriving width, updating active geometry, drawing completed ink, drawing active ink, and completing the render. Instrument both reference and candidate identically; verify that instrumentation does not change the result materially.
- Work per added sample as the active stroke grows. Test approximately 80, 320, 640, 5,000, and 20,000 samples with straight strokes, smooth curves, intentional corners, loops, crossings, and dense scribbles.
- Repaint cost with 0, 5, 25, 100, and 250 long completed strokes. Include many short marks and mixed-tool annotations so stroke count alone is not the only complexity measure.
- The captured centerline and point order. Check corners, start/end positions, loops, and crossings independently of width. Interpolating inside a segment must not be scored as recovering missing real motion.
- Pressure/no-pressure distinctions, stationary pressure changes, timestamp wrap, and all duplicate-input guards.

Implementation work may optimize caching and derived rendering data. This proposal does not preselect a bitmap cache, incremental mesh, or any other renderer design.

## Layer 2: visible real-input replay

Use the actual window and OS input queue with no forced paint between injected events. Replay smooth fast curves, small handwriting-like traces, abrupt direction changes, dots, and pauses at representative mouse rates (125 and 240 Hz, plus a 1000 Hz stress case where the driver is verified). Add real stylus testing with valid pressure and pen history; synthetic pressure tests do not substitute for hardware qualification.

Repeat on empty and crowded canvases and during continuous 5-, 15-, and 60-second strokes. Include mouse, Space-and-mouse, and stylus where available. Check a fresh capture after a dense one, and repeated capture/cancel cycles.

Measure input arrival, application handling, submitted paints, and observed ink presentation separately. Use timestamped frame capture or equivalent presentation tracing to establish visible latency. CPU paint return time alone is not input-to-photon latency. Run the capture mechanism against baseline too and quantify its overhead.

Required display coverage starts with the actual failure configuration (2560 × 1600, 200% scaling) and adds 100%/150% scaling, screenshot and blank-canvas modes, common 1080p/1440p/4K sizes, and multi-monitor coordinate boundaries on available supported hardware. Missing hardware coverage must remain explicit rather than being marked passed.

## Proposed acceptance budgets

These are starting proposals for calibration, not measured guarantees. Keep the budgets fixed before evaluating a repaired candidate; do not raise them to make a candidate pass.

| Measure | Proposed rule |
| --- | --- |
| Baseline non-regression | Per scenario, candidate input/paint/presentation tails must stay within the measured baseline repeatability envelope. Initially flag a repeatable p95 increase beyond `max(1 ms, 10% of baseline p95)`; that allowance accounts for measurement noise, not permission for perceptible slowing. Any visible regression fails regardless of the statistical margin. |
| Absolute interactive work | Target p95 input-dispatch-plus-paint work at or below 16.7 ms, preserving the baseline's faster cadence where available; p99 at or below 33.3 ms. A measured application-induced stall of 100 ms or more fails. Baseline failing an absolute stress budget is a pre-existing problem to resolve or explicitly review, not a reason to relax it for an effect. |
| Input responsiveness | External dispatch p95 at or below 2 ms in the required scenarios, plus no worsening of queue delay/presentation relative to baseline. Instrument handler CPU time separately so blocked dispatch cannot be hidden. |
| Visible ink latency | No worse than the paired baseline. Proposed absolute target: p95 within two display refresh intervals, p99 within three. Capture must establish this before claiming a pass. |
| Path fidelity | No intentional position smoothing or prediction. No extra path loss compared with baseline. Deterministic captured-sample tests must preserve trajectory and corners. Real-input centerline/reference coverage must remain inside the calibrated baseline envelope; a thick-ink mask alone cannot establish fidelity. |
| End of stroke and completion | No delayed beautification pass, tip snap, or hidden backlog after lift-off. Final endpoint, copied output, and completion time must match the on-screen document and not regress from baseline. |
| Resources | No leak across repeated sessions/history operations. Derived caches must be invalidated and released correctly. Record CPU, GPU where available, and private bytes; speed cannot come from unbounded retained state. Preserve the established release-size budget. |

All mandatory scenarios must pass individually. Do not turn workload counts into UI stroke limits. The supported workload envelope, hardware qualification set, absolute budgets, and overload behavior need to be fixed together before this becomes a release gate. It is not a claim that a finite test suite can prevent every delay caused by Windows, drivers, or another process.

## Boundaries against overfitting

A repair fails even if it improves the timings when it does any of the following:

- Smooths, predicts, snaps, rounds, or fits the user's captured path; erases intentional corners or changes small handwriting.
- Silently discards captured input, weakens sampling to hide render cost, caps the number of strokes, removes old marks, or truncates long strokes.
- Uses thicker ink to conceal missing motion or lowers output resolution, antialiasing, or capture quality to improve the score.
- Delays final styling until lift-off, keeps a visible backlog, or moves the slowdown into copy/export or the next capture.
- Flattens away edit/history semantics. Caching pixels is allowed only if the editable document, undo/redo, reset, resizing, clipping, and export remain correct.
- Ignores pressure validity, duplicates pen-generated mouse input, breaks ordinary mouse/Space behavior, or changes arrows, rectangles, and numbered marks.
- Moves unbounded work elsewhere while allowing queues, memory, or background processing to grow without control.
- Optimizes only the exact sine-wave fixture. Qualification includes curves with different frequency, direction, scale, density, and duration, plus unscripted drawing.

Functional checks remain mandatory: undo/redo/reset and branch history; selection resizing and clipping; mouse/Space/pen transitions and capture loss; mixed DPI coordinates; readable dots/short strokes; matching overlay/export; cancellation and clipboard-failure recovery; invocation/completion latency and package size.

## Agreed overload policy

Reject effects that cannot pass the evaluation and retain the accepted baseline until a candidate qualifies. A selected stroke style must behave consistently across workloads. Do not automatically disable effects, change styles, or switch to baseline styling under load. This avoids redundant load-dependent behavior and unpredictable appearance.

Internal rendering caches and incremental updates are allowed when they preserve the same visible result, captured positions, history, endpoints, and matching preview/export. They must not introduce silent path simplification or reduced quality.

The original baseline renderer itself slows at extreme density. Retaining baseline is a release policy, not a claim of unlimited capacity. Repair must address growing rendering work rather than rely on style fallback. The numeric budgets and supported workload envelope remain to be qualified.

## Implementation status

The saved diagnostic harness reproduces the original regression and records evidence. A later [incremental stroke experiment](STROKE-INCREMENTAL-RESULTS.md) implements and evaluates repairs; the user selected Natural and approved committing it after speed/stress testing. The harness is still not a statistically qualified, presentation-aware release gate. No position smoothing or automatic fallback is allowed. User approval of the change does not establish a gate pass: remaining instrumentation, hardware qualification, appearance defects, and proposed budgets are documented in the results.
