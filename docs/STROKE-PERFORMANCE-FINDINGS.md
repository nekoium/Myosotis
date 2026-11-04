# Stroke performance findings

Status: reproduced in the saved experimental binaries. These are diagnostic measurements, not a completed release evaluation. Baseline application source remains unchanged.

## Defined failures

1. **Active-stroke cost grows with stroke length.** A single increasingly detailed stroke can block painting for hundreds of milliseconds. This occurs without historical annotations.
2. **Completed-stroke redraw cost grows with canvas content.** Adding long strokes makes both active and idle forced paints expensive. Caching native geometry does not make redrawing that geometry cheap.
3. **Responsiveness and path fidelity fail together under fast input.** A replayed smooth curve turns into long straight chords in the experimental builds. This is geometric loss of the intended trajectory, not merely staircase aliasing at the pixel edge. The observed cause is consistent with input processing being starved while the UI thread renders; exact OS event loss/coalescing counts have not yet been instrumented.
4. **The original acceptance evidence missed the problem.** Small functional tests and static synthetic exports passed but did not exercise the real input queue or scaling costs of dense native geometry.

Artificial path smoothing is not a remedy for these failures. The user's constraint is to preserve the captured path and fix responsiveness. Brush is retired; future Clean and Natural may differ in width behavior only.

## Reproduction environment

- Existing release executables: Baseline from source commit `892f011b64f6ee075b71f150cff37f69cd4ff535`, and the original Clean/Natural experimental builds.
- Windows native desktop, primary display 2560 × 1600, 200% scaling (192 DPI).
- Selected white canvas: 1400 × 850 physical pixels. No private desktop pixels or clipboard contents were collected.
- Each historical test stroke spans about 1280 physical pixels with 80 prepared moves and a shallow sinusoidal displacement. The counts below mean long strokes, not dots or short dashes.
- All builds use identical deterministic history preparation. Repainting is disabled only during untimed preparation, then enabled for measurements. The pointer is moved and native mouse messages dispatched; an additional real OS `SendInput` replay tests the visible path.
- The revised real-input driver sends 481 positions over two seconds (nominally 240 Hz), with actual injection timestamps saved. Maximum scheduling lateness in the matched third runs was below 0.2 ms. Earlier exploratory runs used a sleep-based driver with about 13 ms of scheduling jitter; they independently showed the regression but are not the matched table below.

Hardware and binary hashes are local artifacts under `.tmp/stroke-eval/`. The diagnostic runner is [Run.ps1](../tools/stroke-eval/Run.ps1), now backed by the [Rust probe](../tools/native/src/probe.rs). The measurements below used the original C# probe from commit `0e0df66`; see the [migration record](RUST-TOOLS-MIGRATION.md).

## Matched results

The following table uses matched third runs, 12 measured cycles per density after five warm-up cycles. Values are nearest-rank p95 wall times in milliseconds; with this small sample size p95 is the maximum observed cycle. They are **forced-paint wall times**, not display presentation latency or an isolated GPU draw-call duration. Pending paint/input work can contribute to the measurement. The real-input curve result independently checks the consequence on screen.

| Prepared historical strokes | Baseline paint p95 | Clean paint p95 | Natural paint p95 |
| --- | ---: | ---: | ---: |
| 0 | 10.9 ms | 21.9 ms | 14.4 ms |
| 5 | 10.9 ms | 156.9 ms | 96.1 ms |
| 25 | 14.0 ms | 706.3 ms | 393.3 ms |

At 25 strokes, input-dispatch p95 was 0.7 ms for Baseline, 344.7 ms for Clean, and 189.1 ms for Natural. These are external synchronous dispatch wall times, including waiting for the application thread; they are not the CPU cost of the handler alone.

Idle forced paints with 25 completed strokes, no active stroke, reached 11.4 ms for Baseline, 588.8 ms for Clean, and 267.7 ms for Natural. This reproduces a completed-content cost even when active geometry is not being rebuilt.

### One growing active stroke

A separate run prepares a continuous Lissajous-like scribble at three lengths, then adds six timed points at each length. No historical strokes are present. Preparation is untimed. Small sample counts make these observed upper tails, not stable production percentiles.

| Approximately prepared motion samples | Baseline paint p95 | Clean paint p95 | Natural paint p95 |
| --- | ---: | ---: | ---: |
| 80 | 11.0 ms | 35.2 ms | 24.7 ms |
| 320 | 10.9 ms | 305.5 ms | 198.2 ms |
| 640 | 11.0 ms | 733.3 ms | 490.9 ms |

### Visible curve fidelity

A two-second, ten-cycle sine wave is replayed above the historical strokes. For each of 460 interior reference positions, the diagnostic measures distance to the nearest coral pixel, up to a 20-pixel search radius. The first and last ten positions are excluded so tapering does not dominate the score.

| Historical strokes | Baseline positions >3 px from ink | Clean | Natural |
| --- | ---: | ---: | ---: |
| 0 | 0% | 2.6% | 0% |
| 5 | 0% | 63.5% | 33.7% |
| 25 | 0% | 91.1% | 88.0% |

Images: [Baseline at 25 strokes](../.tmp/stroke-eval/results/baseline-3/curve-25.png), [Clean at 25](../.tmp/stroke-eval/results/clean-3/curve-25.png), [Natural at 25](../.tmp/stroke-eval/results/natural-3/curve-25.png). The baseline shows the full repeated curve; the experimental builds show only a few long chords.

This image-distance diagnostic can be fooled by overly thick ink and does not directly count processed input samples. It is evidence of this large failure, not a sufficient acceptance metric by itself. Future evaluation must check centerline/sample fidelity and width independently.

### Stress beyond the matched normal-density cases

An earlier baseline-only stress run at 100 and 250 long prepared strokes also degraded. At 250, its forced-paint p95 reached about 132 ms. This does not explain away the new regression: the matched experiments fail much earlier and far more severely. It means that merely matching baseline at arbitrary load is not enough to guarantee responsiveness. The eval proposal therefore includes an absolute usability budget and explicit stress cases. Baseline stress limitations must be recorded and addressed, not used to relax candidate criteria.

## Code-level evidence and remaining uncertainty

The experimental source was inspected in Rim candidate `stroke-clean`:

- `Stroke::push` expands long input gaps into intermediate nodes at a width-relative spacing. Those interpolated nodes do not recover motion that was never captured.
- `Stroke::disks` traverses the entire active stroke whenever its geometry needs rebuilding.
- `Renderer::annotation` rebuilds a changed active path as many overlapping filled quads and circular subfigures, with winding fill.
- `Renderer::image` visits all completed annotations on each repaint and fills their cached path geometries again. The cache stores path definitions, not a rasterized completed-ink layer.
- Mouse handling samples the current cursor position. When processing is delayed, several successive physical positions may no longer be available through ordinary coalesced mouse messages.

These facts explain why the current cache is insufficient and identify active rebuild work and repeated complex filling as the first profiling targets. The experiments have not separated geometry construction, Direct2D tessellation, CPU/GPU filling, presentation synchronization, and queue coalescing with per-stage traces. No claim is made that one API call alone is the complete root cause.

Natural in these measurements is the original frozen build, which contains spatial smoothing. That build is rejected both for its performance and for the newly explicit no-smoothing boundary. It must not be treated as the intended future Natural direction.

## Evidence and limitations

- Raw CSVs and injection records: `.tmp/stroke-eval/results/`.
- Machine details: `.tmp/stroke-eval/machine.json`; binary hashes: `.tmp/stroke-eval/binary-hashes.txt`.
- Aggregated results: `.tmp/stroke-eval/summary.json`, regenerated with `node tools/stroke-eval/Summarize.cjs`.
- At least two exploratory density runs were performed for each experimental direction; the precise-driver third runs supply the matched comparison above. The single-growing-stroke run was performed once per build.
- An initial Clean run was stopped by the command time limit after completing densities 0 and 25; its partial higher-density data is excluded. No Myosotis test process remained running afterward.
- The harness does not measure input-to-photon latency, actual stylus hardware, all display configurations, clipboard export, or leaks across repeated sessions. It does not certify that Baseline or either experiment passes the proposed gate.
- No renderer fix or automatic fallback has been implemented in this investigation.
