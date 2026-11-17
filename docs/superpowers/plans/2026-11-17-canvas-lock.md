# Canvas Lock Implementation Plan

**Goal:** Protect extended drawing from accidental completion with a fixed Shift lock.

**Architecture:** Gate completion input in App, preserve the shared copy path for toolbar buttons, draw the indicator only in overlay paint. Reuse existing Tween and system animation preference without adding settings or a schema action.

**Tech Stack:** Rust, Win32, Direct2D.

**Spec:** [approved design](../specs/2026-11-17-canvas-lock-design.md).

## Constraints and review focus

No added drawing delay or I/O, no desktop input/clipboard automation, no unrelated changes. Verify mouse/pen/Space taps, keyboard exit suppression with rebound keys, repeat suppression, toggling mid-stroke, history, clipboard retry messaging, fade reversal, and export isolation. Shift is already reserved.

## Tasks

- [x] Add App lock/tween state and input gates; preserve pending ink and active strokes; reset per capture; update hints.
- [x] Add vector padlock and overlay-only painting clipped to selection; use existing capture timer.
- [x] Add focused completion tests and memory rendering checks at common DPI.
- [x] Update DESIGN.md/README.md; pass formatting, Clippy, routine tests, release build and offscreen stress.
- [x] Record evidence, limitations, integration notes, and preserve baseline/candidate binaries.
- [x] Commit and push isolated branch, link implementation in #20; report to coordinator on completion.

Execution: this worker implements directly as requested; no helper workers. See delegation result for actual checks and native qualification limits.
