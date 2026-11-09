# Input Schema Transitions Implementation Plan

> Execution: sole worker in the existing issue worktree, as required by the delegated brief. User approved the design, implementation, and finalization. Remaining opening flash is tracked separately as #23.

**Goal:** Soften native schema changes with 160 ms opening, 100 ms closing, and 60 ms control colors.

**Architecture:** Use a schema-only Win32 timer and constant-alpha layered window. Sample short interruptible tweens using elapsed time, and pass blended control-state weights into the existing Direct2D schema renderer. Capture retains the synchronous immediate-hide path.

**Tech Stack:** Rust, existing windows crate, Win32, Direct2D. No new dependencies.

**Spec:** `docs/superpowers/specs/2026-11-08-input-schema-animation-design.md`

## Constraints

Preserve #14 behavior and #15 geometry. Respect Windows client-area animation preference. No screenshot/annotation animation, blocking waits, desktop-input tests, or idle timers. Preserve baseline and candidate executables. Do not merge before user testing approval.

## Review focus

Rapid reversal must start at the displayed opacity/color. Repeated close must not restart dismissal. Capture must cancel timers and hide immediately. Disabled animations must settle all values. Dismissal must reject input and never restore focus.

## Tasks

- [x] Preserve release baseline from `53d070a` under `.tmp/issue18/baseline/` before editing production code.
- [x] Add `src/windows/schema/motion.rs`: elapsed-time tween with exact preview easing, interruption, and settle tests.
- [x] Integrate schema visibility/control tweens in `src/windows/schema.rs`: timer only while active; reduced-motion query and setting-change handling; immediate capture hide; reopening and input gating. Add invisible-window lifecycle tests.
- [x] Update `src/windows/render.rs`: interpolated state weights for schema controls, with existing static offscreen rendering unchanged. Add representative offscreen blended-color checks in `src/windows/render_tests.rs`.
- [x] Update DESIGN.md and approved spec. Record decision and medium scope in GitHub when network access succeeds.
- [x] Run formatting, Clippy, unit/offscreen tests, release build, and offscreen stroke stress. Preserve candidate under `.tmp/issue18/candidate/` and record comparison instructions and remaining live qualification.
- [x] Follow-up requested by user: restore 160 ms entrance, use all non-control schema space as native drag area, and toggle schema with tray left-click. Preserve press intent through focus-loss dismissal.
- [x] Investigate flash: separate show from first-frame paint, hold alpha while rendering, begin tween afterward, and release schema-state borrow around synchronous ShowWindow/focus calls. Add offscreen state/order and interaction regression checks; qualify visible flash separately through user testing.

User comparison approval was received for the current changes; remaining flash investigation continues in #23. Rim saves intermediate work.
