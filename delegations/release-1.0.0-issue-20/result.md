# Issue #20 result

Outcome: implemented and automated checks passed; native interaction confirmation remains outstanding.

## User decisions

The user approved Shift as a press-once lock toggle and clarified that only the quick toolbar Copy/Cancel buttons remain available while locked, not keyboard shortcuts. Locked mouse double-clicks, pen double-taps, and repeated Space taps draw normally. Enter and configured Copy/Cancel keys are disabled. A black toolbar-style vector padlock inside the selected field's top-right corner fades in, remains while locked, and fades out when unlocked; it is not exported. Each capture starts unlocked; no persisted preference or rebindable action was introduced.

Confirmed decision posted directly at the user's explicit instruction: https://github.com/nekoium/Myosotis/issues/20#issuecomment-6952698085. That later instruction superseded the dispatch's earlier GitHub-posting restriction.

## Implementation

Isolated branch `nekoium/release-1.0.0-issue-20`, based on origin/master `47b6fd6e3cc7794c1307f8acfacd17c17aadb91a`. Fixed Shift remains reserved under existing binding validation. App gates gesture and keyboard completion; toolbar routes retain the existing copy/recovery path. Toggling commits pending visible click ink, preserves active strokes, and resets gesture recognition. Locked taps commit immediately. History/editing/resizing remain available.

Existing Tween provides 160 ms fade-in / 100 ms fade-out with continuous reversal, respecting Windows animation preference. Existing capture timer repaints only while the fade is unsettled; indicator rendering is overlay-only, clipped to selection bounds. Locked hints omit inactive keyboard completion instructions and clipboard retry hints direct users to the toolbar.

## Actual checks

- `cargo fmt --check`: passed.
- `cargo clippy --all-targets -- -D warnings`: passed.
- `cargo test --locked`: 47 passed, 2 intentionally ignored.
- `cargo build --locked --release`: passed.
- `cargo test --locked --release offscreen_stroke_stress -- --ignored --nocapture`: passed (1 test).
- `git diff --check`: passed.
- Focused tests cover Shift repeat suppression, pending ink/history, active Space stroke preservation, mouse/pen/Space taps, fixed and rebound keyboard Copy/Cancel suppression, toolbar Cancel, reset, and lock-aware retry hints. Existing clipboard tests continue passing.
- Memory rendering tests cover opacity, bounds/clipping, and export isolation at 100%, 125%, 150%, 200% DPI. Existing tween tests cover fade reversal and settling.

The first memory rendering assertion assumed individual stroke opacity also bounded overlapping strokes; it was corrected to check total darkness and nonzero partial-opacity values. Final checks above passed.

## Limitations and retained binaries

No visible window, pointer/focus automation, system clipboard replacement, or real stylus test was performed. Automated evidence does not establish perceived input latency, visible fade quality, real hardware double-tapping, toolbar Copy/clipboard contention in the running app, or baseline/candidate interactive parity. Do not describe these as qualified.

Runnable binaries retained locally (ignored by git) for a future explicitly authorized comparison:

- `.tmp/canvas-lock/baseline.exe`: source `47b6fd6`; SHA-256 `59ddcc233a2c6d683da4fc1e848bd8d27e0c6a032d7fb566e8ebeb147773431b`.
- `.tmp/canvas-lock/candidate.exe`: implemented candidate; SHA-256 `1a568c509d6cc645673e3f7071b6b8c9c874a4b3e3edb09da24948c84324833f`.
- `target/release/myosotis.exe` is the candidate. Neither binary was launched.

## Integration notes

Shared files: `src/windows/mod.rs` (App state/lifecycle, input gates, hints, overlay paint argument, timer), `src/windows/render.rs` (new icon index 11, indicator helper, paint opacity argument), `src/windows/schema.rs` (only `pub(super)` visibility for motion and animations_enabled), DESIGN.md, README.md. Input schema layout, actions, rebinding model, settings, clipboard implementation, release versions, and other workers' branches were not edited.

Apply after #23 and before #25 per coordinator plan. Preserve #23 presentation lifecycle changes when merging the two visibility-only schema lines. #25 must retain lock state initialization/reset, lock-aware copy-failure messaging, and the existing toolbar Copy route while adding local save. No dependency on #25 changes. Resolve docs hunks by retaining both features. No master merge or release publication performed.

Issue text: `issue-reply.md` (posted confirmed decisions), `issue-body.md` (posted final problem/decision/checklist), and `implementation-reply.md` (posted final implementation evidence).

Implementation commit: [`03c5794`](https://github.com/nekoium/Myosotis/commit/03c5794), pushed to `origin/nekoium/release-1.0.0-issue-20`. Implementation/evidence comment: https://github.com/nekoium/Myosotis/issues/20#issuecomment-6953071044. The issue remains open for native confirmation and integration review. A final documentation commit records these references and the posted text; both commits are synced.
