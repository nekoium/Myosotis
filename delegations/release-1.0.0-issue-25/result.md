# Issue #25 result

Outcome: succeeded — user-approved feature implemented, committed and pushed to `origin/nekoium/release-1.0.0-issue-25`.

Implementation commit: `00aec458e1eb9c40c57de74112250cb077ae4103` — Add optional background PNG saving from tray (#25).

## Approved decisions

Persistent **Save to local**, default off, immediately below **Start at login**. Each successful clipboard completion while enabled saves the exact flattened image as PNG under Windows Pictures/Screenshots. Local timestamp names follow `Screenshot 2026-11-15 215736.png`; collisions append numbered suffixes. Canceled or initially failed copies save nothing. Encoding/file I/O run in workers after hiding the overlay; errors preserve clipboard output and defer visible reporting while a subsequent capture is open. Normal Quit waits for saves. **Dark Theme** renamed **Dark theme**.

User explicitly authorized posting the confirmed decision despite the original dispatch restriction; posted https://github.com/nekoium/Myosotis/issues/25#issuecomment-6952374179 before implementation. User then requested commit and sync; pushed issue branch only. No further GitHub posts, PR, release/version changes or merge performed.

## Files and integration

- `src/windows/mod.rs`: new module/imports/constants, App preference/job fields, successful-copy owned snapshot dispatch, save error/preference helpers, tray checkbox and capitalization, save timer handling, startup preference loading, shutdown job drain. No input/schema binding/layout edits. Timer ID 2; other workers must avoid reusing it.
- `src/windows/clipboard.rs`: Image caches one WIC PNG result with OnceLock; Publication retains Arc<Image>, allowing saving independent of clipboard replacement without duplicating compression. Existing delayed clipboard message handling and shutdown retained.
- `src/windows/local_save.rs`: Windows Pictures known folder, timestamp, create-new collision handling, file-write/sync cleanup, worker management.
- `src/settings.rs`: isolated save_to_local boolean preference through existing TOML preservation/replacement helpers.
- `Cargo.toml`: enable Windows SystemInformation for GetLocalTime; no new crate/version change, lockfile unchanged.
- `src/windows/{local_save_tests,clipboard_tests,completion_tests}.rs`: focused coverage.
- `tools/Test-Themes.ps1`: seven menu items / renamed theme / new checkbox / Quit index expectations.
- `DESIGN.md`: menu description and Save to local section; `README.md`: usage/settings/privacy; `docs/VALIDATION.md`: evidence and limits.
- Local brief, approved design/plan, issue decision reply and this result under `delegations/release-1.0.0-issue-25/`.

Provisional coordinator integration order remains #23, #20, #25. Cherry-pick implementation commit and resolve additive App initializers/imports/mod.rs message handling as needed; #20 may add App fields in completion tests, and documentation may overlap. No other branches merged. Result/checklist follow-up commit accompanies implementation on the branch.

## Actual checks

- `cargo fmt --check`: passed.
- `cargo clippy --locked --all-targets -- -D warnings`: passed.
- `cargo test --locked`: earlier parallel run passed (47 passed, 2 ignored before added caching test); final parallel run had 47 passed, 1 existing schema-recording test failure, 2 ignored.
- `cargo test --locked dismissal_clears_recording_and_focus_and_reset_requests_only_defaults -- --nocapture`: passed after that failure.
- `cargo test --locked -- --test-threads=1`: passed, 48 passed and 2 intentionally ignored.
- `cargo test --locked --release offscreen_stroke_stress -- --ignored --nocapture`: passed, 1 passed, 48 filtered. Initial combined command timed out during release compilation; standalone diagnostic completed successfully.
- `git diff --check` and staged check: passed.
- Focused tests cover persistence/malformed settings/save failure preservation, concurrent file collisions, encoding and destination errors, cancel/unselected no-save behavior, shared cache under concurrency, source release/shutdown, and hidden-window PNG decode checking selected bounds/annotation pixels/opacity/screenshot and white backgrounds. Synthetic files stay under project .tmp and clean themselves up.

## Limits / remaining release integration

Existing schema test reads live keyboard state; intermittent Escape recording assertion at schema.rs:961 passed isolated and serial reruns. No schema code changed; report during combined release qualification rather than treating a serial pass as native qualification.

No desktop/focus-driving suite or system clipboard replacement test was authorized or run. Native tray interaction, error dialogs, redirected Pictures folders, actual application paste compatibility and long-session/real-input responsiveness remain unqualified. No Save As dialog exists in approved workflow. Forced process termination cannot guarantee pending saves; normal Quit drains workers. A failed write attempts to remove incomplete output; cleanup itself can fail if storage becomes unavailable. Further issue checklist/implementation-link update should be published by coordinator using this result; issue stays open pending integration.
