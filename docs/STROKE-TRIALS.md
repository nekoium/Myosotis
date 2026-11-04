# Stroke trials and performance investigation

Work for [annotation appearance, issue #4](https://github.com/nekoium/Myosotis/issues/4). This document records the original failed experiments. The later incremental Natural implementation was selected and approved by the user; see [the current results and approval](STROKE-INCREMENTAL-RESULTS.md). Blazing-fast responsiveness takes precedence over appearance.

## Current incremental experiments

A later implementation and self-evaluation are recorded in [incremental stroke results](STROKE-INCREMENTAL-RESULTS.md). Use `tools/Try-StrokesV2.ps1` for those frozen builds. The user selected Natural and approved the speed/stress testing; standard builds now enable Natural. The original baseline executable and old failed experiments are preserved. The baseline-only status below describes the earlier diagnostic phase.

## Original diagnostic status

Interactive diagnostics reproduced the reported regression in both Clean and Natural. Long active strokes become increasingly expensive. Accumulated completed strokes also make redraws slow. Replaying the same fast curve produces long straight chords in the experimental builds where baseline follows the curve.

See [the measured findings](STROKE-PERFORMANCE-FINDINGS.md) and [the proposed evaluation and boundaries](STROKE-PERFORMANCE-EVAL.md). The baseline remains the application source in the active evaluation candidate. No performance fix has been implemented.

**Brush is retired from the comparison lineup.** Its historical Rim candidate is preserved. The existing experimental binaries are frozen evidence, not approved new builds: their F6 cycle still includes Brush, and the old Natural build still uses path smoothing. Do not use F6 during performance comparisons.

**Artificial path smoothing is excluded from future directions.** Clean and Natural may change stroke width; they must preserve the captured path. Missing motion caused by stalls must be fixed at input/rendering performance, not hidden with smoothing or prediction.

## Builds available for diagnosis

| Build | Application behavior | Status |
| --- | --- | --- |
| Baseline | Exact application source from release `892f011b64f6ee075b71f150cff37f69cd4ff535` (0.1.2), original fixed-width drawing | Reference |
| Clean | Restrained endpoint taper and pressure response | Known performance failure |
| Natural | Mild speed-based width variation and pressure response; old build also smooths positions | Known performance failure; future version must remove position smoothing |

The executables are under `.tmp/stroke-trials/baseline`, `clean`, and `natural`. The baseline was rebuilt from source, not copied from an unverified old executable. The launcher preserves any existing resident app and refuses to silently activate it in place of the requested build.

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools/Try-Strokes.ps1 -Direction Baseline
```

Finish or cancel your capture and quit the running Myosotis through its tray menu before changing builds. Replace `Baseline` with `Clean` or `Natural` for a separate run. Each command opens capture immediately. Right-drag a white canvas, then press `1` for pen. Use the same canvas size, display scaling, and input device. Start fresh for each test. Do not enable Start at login for temporary executables.

## Diagnostic replay

For routine rendering work, first use the cursor-free checks in [CONTRIBUTING.md](../CONTRIBUTING.md#check-changes), including the optional `offscreen_stroke_stress` test. The native replay below requires exclusive desktop use throughout execution; it is not suitable while the user works at the computer. Run it only in an explicitly available desktop or separate interactive Windows environment.

The diagnostic harness drives the existing executables on a white fixture. It moves the pointer, takes focus, captures only the selected white test canvas, and leaves the clipboard untouched. It refuses to run if Myosotis is already resident, and closes only the process it starts. It restores the previous foreground window and pointer position on ordinary completion or failure. Avoid interrupting it; forcibly terminating the harness may bypass cleanup.

```powershell
powershell -NoProfile -STA -ExecutionPolicy Bypass -File tools/stroke-eval/Run.ps1 -Direction Baseline -Repeat 10 -Interactive
powershell -NoProfile -STA -ExecutionPolicy Bypass -File tools/stroke-eval/Run.ps1 -Direction Clean -Repeat 10 -Interactive
powershell -NoProfile -STA -ExecutionPolicy Bypass -File tools/stroke-eval/Run.ps1 -Direction Natural -Repeat 10 -Interactive
node tools/stroke-eval/Summarize.cjs
```

Use a fresh repeat number to retain earlier results. Default diagnostic densities are 0, 5, and 25 long historical strokes, with 12 measured forced-paint cycles each. To test a single growing stroke:

```powershell
powershell -NoProfile -STA -ExecutionPolicy Bypass -Command "& ./tools/stroke-eval/Run.ps1 -Direction Baseline -Densities @(-1) -Repeat 11 -Interactive"
```

Replace `Baseline` to compare the other builds. The single-stroke probe checks approximately 80, 320, and 640 prepared input samples, then adds a few samples while timing paint work. Preparation is untimed and display is suppressed equally for all builds; display is enabled for every measurement and real input replay.

Results go to `.tmp/stroke-eval/results/<build>-<repeat>/`. CSVs record input dispatch, forced paint wall time, private bytes, actual replay injection timestamps, and reference-curve coverage. PNGs show the visible result. The script is an exploratory diagnostic, **not yet the proposed release gate**: small sample counts, forced paints, and screenshot-distance checks do not prove presentation latency, input retention, export correctness, or hardware stylus quality.

## Manual checks

Draw fast strokes on a fresh white canvas, then test one continuous 15–20 second scribble. Separately fill a new canvas with many individual lines and repeat the initial strokes. Check whether a fresh capture restores responsiveness. Repeat with mouse, Space-and-mouse, and a real stylus when available.

After performance is acceptable, check small writing, intentional corners, dots, short dashes, loops, pressure changes, undo/redo, resizing, and copied output. Neither losing detail nor smoothing the path counts as a performance fix.

## Rim

The saved `stroke-clean`, `stroke-natural`, and retired `stroke-brush` candidates retain the original experiments. `stroke-baseline` restores the pre-change application. `stroke-performance-eval` holds the diagnostic tools, findings, and proposed eval, with baseline application source. Save hot changes before switching. `.tmp/stroke-trials/candidates.json` identifies the saved trial nodes.
