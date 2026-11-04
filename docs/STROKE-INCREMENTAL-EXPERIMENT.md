# Incremental stroke experiment

## Authorization and scope

The user delegated direction selection, implementation, and evaluation, then selected Natural and approved committing it after speed/stress testing. Natural is now the default profile. Preserve the existing baseline and frozen failed binaries. Do not change style under load.

## Design

Compare Clean (endpoint taper, pressure) with Natural (the same plus mild mouse-speed width response). Store captured coordinates unchanged; keep width derivation independent. Width filtering never filters positions. Use small independent segment geometries and circles, not one overlapping winding-filled path. Keep completed ink and the stable active prefix in a transparent compatible bitmap. Redraw only the uncommitted trailing taper over that bitmap. The full frame restores the original background first, so shrinking tips leave no stale pixels. The retained document remains authoritative for history and uncached export.

A fixed start ramp and fixed trailing distance make old widths stable. Dots retain their full radius; tiny strokes retain a minimum readable radius. No lift-off restyling. Pressure takes precedence over simulated mouse speed. Both profiles use identical rendering, input, and caching code.

## Plan

1. Add sample/width data and local taper calculations; test corners, duplicates, pressure validity, wrap, and prefix stability.
2. Add append-only native bitmap caching with explicit document history invalidation; preserve uncached export and selection clipping.
3. Add pen pointer history, pressure-mask handling, promoted-mouse suppression, and capture-loss cleanup. Preserve mouse/Space behavior.
4. Build both profiles into separate experimental folders; retain baseline and old failed builds. Run formatting, clippy, unit tests, native functional checks, and matched real-input replay at multiple densities and active lengths.
5. Inspect images, timings, export agreement, and resource behavior. Record failures and unavailable hardware honestly. Do not claim the full presentation-aware release gate passed.

## Outcome

Implemented and tested both profiles with incremental primitive rendering. Also tested and rejected a small-batch winding-path alternative. The evaluator preferred Clean; the user selected Natural and approved its speed/stress testing. Natural is enabled by default, with visual seams and incomplete qualification documented as remaining limitations. See [the full results and approval](STROKE-INCREMENTAL-RESULTS.md).

## Validation boundaries

No smoothing, prediction, discarded captured trajectory, stroke caps, quality reduction, automatic fallback, delayed final styling, or history flattening. Bitmap memory scales with canvas area; document memory scales with retained samples. History rebuild and export costs require measurement. Real stylus hardware is required to qualify actual pen behavior; synthetic samples cover logic only.
