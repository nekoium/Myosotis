Implemented and pushed in [03c5794](https://github.com/nekoium/Myosotis/commit/03c5794), branch [`nekoium/release-1.0.0-issue-20`](https://github.com/nekoium/Myosotis/tree/nekoium/release-1.0.0-issue-20).

Shift now toggles lock once per press. Locked mouse/pen/Space taps draw without completing, and Enter plus configured Copy/Cancel shortcuts are disabled; the quick toolbar Copy/Cancel buttons remain available. The black padlock uses the toolbar vector style, fades in/out at the selected area's top-right, stays visible while locked, respects Windows' animation preference, and is excluded from copied pixels. Each capture starts unlocked. Input schema/settings remain unchanged because Shift was already reserved.

- [x] Input gates, active-stroke/pending-ink preservation, lock-aware hints and reset.
- [x] Focused input/history and memory rendering checks at 100%, 125%, 150%, 200% scaling.
- [x] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --locked` (47 passed, 2 intentionally ignored), release build, and explicitly run release `offscreen_stroke_stress` passed.
- [x] DESIGN.md and README.md updated; runnable baseline/candidate binaries retained locally.
- [ ] Native interaction confirmation: real pen/mouse double-tapping, visible fade, toolbar Copy/clipboard recovery and baseline/candidate responsiveness.

No desktop input/focus/clipboard automation was run. The branch is ready for integration review; this issue remains open for native confirmation. Full evidence and shared-file integration notes are in `delegations/release-1.0.0-issue-20/result.md`.
