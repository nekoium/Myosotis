# Version 0.2.0 validation

## Package

Cargo and Windows file/product resource versions are `0.2.0`. Natural is enabled by default. The version change does not modify stroke/input/rendering behavior from approved commit `0e0df66` or the separate palette change `fbbb4d7`.

- Executable: 571,392 bytes.
- ZIP: 360,758 bytes.
- Installed portable folder: 663,392 bytes.
- ZIP SHA-256: `d155191ab074dd40518b55261febb0792cc693d3f1fdbb3f05929eca528c33fd`.
- Packaged executable SHA-256: `8ce178ef21b6344dbb3ac91e1d6fc106b987712adc680844625d5fef7700d6fb`.
- The package contains the executable, README, MIT license, third-party notices, and the two current README screenshots. No local captures, settings, symbols, or build tools are included.
- The Scoop manifest points to the prepared 0.2.0 ZIP and matches its checksum. The URL becomes valid after publication.

## Checks

- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test --locked`: 20 unit tests passed.
- Windows x64 MSVC release build and package size checks passed.
- The final packaged executable passed the full native smoke suite and theme suite. Smoke covered capture/completion, all tools, Space, undo/redo/reset, resizing, clipboard contention/retry, bitmap/PNG pixels, foreground restoration, repeated invocation, and shutdown.
- Packaged smoke ended at 75.4 MiB working set and 269 handles. This does not establish leak freedom.
- Synthetic Windows pen injection produced visibly different light/heavy/increasing-pressure strokes. Preview/export had 6 ink-mask differences, none more than one pixel from corresponding ink; 6,767 exact color-pixel differences remain.
- Native light/dark captures use isolated temporary settings and a synthetic scene. Original icon geometry is preserved. The former resident executable was closed only while hidden, then restored. Clipboard formats were backed up around smoke/theme/export checks.

The first smoke attempt stopped after its first check with a PowerShell null-value error; an immediate full retry passed, and the packaged executable then passed the full suite independently. A first 60-second replay was interrupted by the command tool's 60-second timeout and has no scored fidelity result. It is excluded. A replacement ran in a separate process to completion; only that completed run is reported.

## Fresh performance comparison

Evidence: [performance.json](performance.json), [frame observations](performance-frames.csv), and [coverage results](performance-fidelity.csv). Five alternating runs per build, 24 timed frames after five warmups per density per run. Each density also receives a two-second 240 Hz real-input curve replay. The canvas is 1400 × 850 physical pixels on a 2560 × 1600 display at 200% scaling (192 DPI).

| Historical strokes | 0.1.2 p95 paint | 0.2.0 p95 paint | Observations per build |
| --- | ---: | ---: | ---: |
| 0 | 10.754 ms | 10.672 ms | 120 |
| 25 | 13.784 ms | 10.781 ms | 120 |
| 100 | 49.023 ms | 10.812 ms | 120 |
| 250 | 132.969 ms | 10.899 ms | 120 |

All 9,200 checked reference positions in 0.2.0 were within three pixels of ink. Baseline missed 1,085 of 2,300 at 250 strokes. These are paint-return wall times and final image-mask checks, not presentation latency or rigorous sample accounting.

The matched benchmark and showcase-capture executable was built in an isolated Cargo target folder. Its SHA-256 is `bcd983ec4f3515ec83fc3d346210c57a29bd9113d0aef11422b4ee88a56ea48e`. The package was built through `tools/Package.ps1` from the same source and features in the standard target folder. The hashes differ; they are not asserted to be identical artifacts. The exact packaged executable received its own smoke/theme suite and completed 60-second replay.

The packaged executable's 60-second 240 Hz replay over 25 historical strokes had zero missed positions out of 14,380 checked, with maximum injection lateness 0.619 ms. This confirms final coverage, not input-to-display latency.

Single-run secondary diagnostics on the isolated 0.2.0 build:

- 20,000 prepared active samples: first catch-up paint 37.167 ms; subsequent five paints at most 10.825 ms.
- 250-stroke undo/redo/reset restoration: up to 66.500 ms paint time.
- 250-stroke synchronous copy: 108.307 ms external dispatch; application logged 89 ms clipboard completion.

These secondary observations do not establish production percentiles. Copy still exceeds the proposed 100 ms boundary. The user's approval of Natural remains recorded; this release preparation does not claim the full proposed gate passed.

## Image provenance

The palette session's workflow and component studies under `.tmp/annotation-colors/` informed layout and coverage. The stroke session's frozen builds, results, and no-smoothing/consistent-style decisions informed the comparison. The new showcase uses native captures instead of those schematic palette previews.

- `01-before-after.png`: same injected positions and synthetic content in frozen 0.1.2 and current 0.2.0. Scene views and detail crops use equal scales. Event scheduling can vary between runs; this is not a timing comparison.
- `02-components.png`: actual 0.2.0 dark/light toolbar states, guidance, annotations, and synthetic pen output. No substitute icon drawings or mock native menus.
- `03-stress-test.png`: native 250-stroke images from the first fresh comparison run, measurements aggregated across all five runs, and the completed packaged 60-second replay result. The dense bands are the deliberate overlapping history fixture, not ordinary handwriting.

Full PNGs were inspected after composition. All scene content is synthetic. Local capture/log/input artifacts remain under `.tmp/release-0.2.0/` and `.tmp/stroke-eval/results/release-020-*/`. The release kit includes the final PNGs and public benchmark CSVs.

## Reproduction

Quit the resident app before native checks. These commands control mouse and focus. The clipboard-preserving wrapper below protects formats around smoke/export checks; `tools/Test-Themes.ps1` itself replaces clipboard contents.

```powershell
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --locked
powershell -NoProfile -ExecutionPolicy Bypass -File tools/Package.ps1
powershell -NoProfile -STA -ExecutionPolicy Bypass -File tools/stroke-eval/Check-Functions.ps1 -Exe dist/Myosotis/myosotis.exe -Interactive
powershell -NoProfile -STA -ExecutionPolicy Bypass -File tools/Test-Themes.ps1 -Exe dist/Myosotis/myosotis.exe -Interactive
```

`tools/release/Capture-Showcase.ps1` writes native fixture captures with isolated theme settings. `tools/release/Summarize-Performance.cjs` validates and aggregates the named local release runs. `tools/release/Build-Showcase.ps1` composes all three PNGs from those captures and measurements. They require the preserved local source artifacts; their filenames are explicit in the scripts. The chart uses the standard .NET chart library; raster composition uses System.Drawing.

## Remaining limits

Small antialias seams, nonidentical preview/export pixels, real stylus qualification, presentation latency, 1000 Hz input, long-duration resource tracing, and broad mixed-DPI/multi-monitor testing remain unresolved. Package publication, signing, an installer, macOS, and Linux are outside this preparation. Existing startup registration was not changed.

Rim candidate `release-0-2-0-prep` (`n357666f4f364f46`) preserved the starting workspace after `rim status` again reported no root. No Rim state was purged or reinitialized. Unrelated CONTRIBUTING, AGENTS, delegation, and capture-file changes remain outside the release commit.
