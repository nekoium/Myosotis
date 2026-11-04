# Coral/periwinkle implementation validation

Approved design: [palette and badges](superpowers/specs/2026-11-04-annotation-palette-design.md). Related product problem: [issue #4](https://github.com/nekoium/Myosotis/issues/4).

## Implemented

- Annotation ink `#C74656` in both the incremental overlay ink cache and uncached export renderer.
- Interface accent `#5865D8` for Copy, selection borders, and resize handles.
- Segoe UI semibold 14-DIP number labels; 24-DIP badge height and adaptive width with 8-DIP side padding measured from visible bounds.
- DirectWrite overhang metrics center the visible numeral group. Layouts are cached per number for the renderer lifetime and reused during export.
- Existing icon method, toolbar geometry, neutral theme surfaces, application icon assets, and input behavior are preserved. Existing stroke implementation and caching are preserved.
- Native smoke/theme pixel checks now recognize the refined coral value.

## Checks passed

- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test --locked`: 19 passed.
- `cargo build --locked --release`
- `tools/Smoke-Test.ps1`: full native capture, annotation, history, resize, completion, clipboard contention/recovery, DIB/PNG pixels, frozen capture, foreground restoration, repeated invocation, and shutdown suite passed.
- `tools/Test-Themes.ps1`: default settings behavior, menu marks, persistence, live palettes, selection/history preservation, and white-canvas/coral export passed with isolated settings.
- New offscreen Direct2D/DirectWrite test checks rasterized white numeral bounds, visible coral fill, horizontal padding, and multi-digit expansion. It covers 1, 4, 7, 10, 11, 20, 100, and the largest `usize`, at scales 1, 1.25, 1.5, and 2. Raster centers are within one physical pixel of the requested center.
- Git whitespace check passed. Icon geometry has no changes, and Rim's comparison reports no asset changes.

## Limits

These checks do not establish interactive long-stroke latency or performance acceptance for the pre-existing stroke experiments. No pen geometry or caching algorithm was changed here.

An additional native preview attempt was blocked by the preview tool's existing-resident check: `.tmp/stroke-v2/natural/myosotis.exe` was running. That process was left untouched. Temporary isolated settings for this failed preview were removed. Native workflow and theme checks above had already completed successfully.

No release, package, Git commit, or external issue update was published.

## Workspace checkpoint

Rim candidate `coral-periwinkle` (`n6765846c3add5a8`) captured the starting workspace and the implementation checkpoint. `rim status` initially reported no root; `rim spawn` successfully created the candidate with a baseline preserving the existing dirty workspace. `rim diff` supplies file-level changes rather than a textual diff, so Git diff and source inspection were also used. Concurrent edits to `tools/stroke-eval/Probe.cs` were observed after the implementation checkpoint and preserved.
