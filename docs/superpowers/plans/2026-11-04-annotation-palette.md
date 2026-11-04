# Annotation Palette Implementation Plan

> Execution: implement directly in the current session, as authorized by the user. Preserve all existing workspace changes.

**Goal:** Apply approved coral/periwinkle colors and centered adaptive number badges without changing icons.

**Architecture:** Keep the shared native overlay/export renderer. Cache DirectWrite number layouts and measure visible bounds for placement. Retain existing stroke caching.

**Tech Stack:** Rust, Win32, Direct2D, DirectWrite; no new dependencies.

**Spec:** `docs/superpowers/specs/2026-11-04-annotation-palette-design.md`

## Constraints and review focus

- Exact ink `#C74656`, accent `#5865D8`; neutral UI surfaces remain unchanged.
- Preserve the `icon` method and image assets byte for byte relative to the pre-change workspace.
- Check single and multiple digits at 100%, 125%, 150%, and 200% scale.
- Verify actual raster bounds, padding, and alignment rather than only recalculating the implementation's formula.
- Overlay and export share badges and ink; no measurement in the pen hot path.
- Existing stroke work is outside scope. Interactive responsiveness is not proven by unit tests.

## Tasks

- [x] Update `src/windows/render.rs`: shared palette constants; cached `NumberBadge` layouts using Segoe UI semibold 14 DIP, 24-DIP badge height, 8-DIP side padding, visible-bound centering; preserve icons.
- [x] Add native offscreen raster tests for centered digits `1`, `4`, `7`, `10`, `11`, `20`, `100` and large numbers at four scales. Check measured white pixel bounds and coral badge pixels.
- [x] Update coral pixel predicates in `tools/Smoke-Test.ps1` and `tools/Test-Themes.ps1` for the approved ink. Record the decision and validation scope in `DESIGN.md` and a dedicated validation note.
- [x] Run formatting, Clippy, tests, and release build. Compare icon method/assets against the Rim parent; inspect final diff and preserve a Rim checkpoint.

GitHub issue updates require no action during this local implementation: the user authorized implementation, not external messages. Include issue links in local documentation.
