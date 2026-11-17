# Combined local candidate result

Outcome: succeeded — #20 and #25 integrated locally, reviewed, checked offscreen, and exported for user confirmation. No runtime/native qualification was performed. No master merge, push, tag, release publication, issue update/closure, or WinGet submission was performed.

## Source and integration

Branch: `nekoium/release-1.0.0-candidate`, based on `origin/master` at `47b6fd6e3cc7794c1307f8acfacd17c17aadb91a`.

- #20 original `03c579403c7ab9f4bbf3bb1b9a9062a272fb9ee9` cherry-picked as `3b98b43`: fixed Shift lock, blocked keyboard/gesture completion while locked, retained toolbar Copy/Cancel and retry, overlay-only fading black padlock.
- #25 original `00aec458e1eb9c40c57de74112250cb077ae4103` cherry-picked as `9dcaa4d`: persistent default-off Save to local below Start at login, Pictures/Screenshots PNGs, create-new collision handling, background encoding/I/O, normal Quit draining jobs, Dark theme capitalization.
- Both cherry-picks applied without textual conflicts. Git correctly combined App production/test initializers, imports, completion-success snapshot handling, lock-aware failure hints, timer branches, README, and DESIGN. No manual code conflict resolution or new product behavior was needed.
- Reviewed connected input/toolbar paths, lock state reset, render/export separation, App initialization, timer routing, settings persistence, local-save workers, shared OnceLock PNG cache, clipboard ownership/retry/publication, and shutdown. Capture/lock animation uses timer 1, local save timer remains 2, clipboard timer remains `0x504e47`; schema uses timer 18 on its own window. Clipboard Publication retains the owned Arc snapshot until the overlay is hidden; background local saving remains independent of subsequent clipboard replacement. Normal shutdown publishes pending clipboard PNG while its owner exists, then drains local-save jobs after the message loop.
- #23 had no implementation commit. Read the other worktree's uncommitted DESIGN diff and transferred only its disposition sentence, preserving #20's adjacent indicator exception. Commit `430c06503e395796b2b722d142be5eae74004257` records that sentence and this task brief. User could not reliably reproduce after refreshed `47b6fd6` and requested closure as not planned; no flash correction or root cause was confirmed.

The exported executable was built from the combined implementation at `9dcaa4d`; source record `430c06503e395796b2b722d142be5eae74004257` has identical application/build code and adds documentation only. Subsequent candidate evidence commits likewise change documentation only. Existing issue worktrees, binaries, and the original dirty main checkout were not modified.

## Actual checks

All commands ran in this isolated worktree on the Windows x64 MSVC host (`rustc 1.99.0`, host `x86_64-pc-windows-msvc`).

| Check | Actual result |
| --- | --- |
| `cargo fmt --check` | Passed |
| `cargo clippy --locked --all-targets -- -D warnings` | Passed |
| `cargo test --locked` | Passed: 54 passed, 0 failed, 2 intentionally ignored; default parallel run, 21.47 s test execution |
| `cargo build --locked --release` | Passed; 45.45 s |
| `cargo test --locked --release offscreen_stroke_stress -- --ignored --nocapture` | Passed: 1 passed, 55 filtered; 0.73 s test execution |
| `git diff --check` | Passed before source-record commit and for final evidence |

Routine tests include lock repeat/input gates/history/reset, toolbar Cancel, retry hints, indicator opacity/clipping/export exclusion at common DPIs, exact local PNG bounds/background/annotation decode, concurrent filename collisions, save errors, settings preservation, shared cached encoding, source-release/shutdown jobs, existing clipboard worker/memory checks, schema memory rendering, and stroke cache/history coverage. The system-clipboard-changing lifecycle test remained ignored; the separate release stress diagnostic was explicitly run.

Release stress (diagnostic, not an interactive or numeric performance gate):

| Completed history | Cache setup | 20,000 samples + cache updates |
| --- | --- | --- |
| 0 | 462 µs | 170.888 ms |
| 25 | 12.2939 ms | 153.6642 ms |
| 250 | 108.5321 ms | 151.9033 ms |

#25's worker recorded an earlier final parallel run with 47 passed, 1 failed (existing `dismissal_clears_recording_and_focus_and_reset_requests_only_defaults`, Escape assertion at schema.rs:961), and 2 ignored. Its isolated rerun passed, and serial full suite passed 48 with 2 ignored. The schema recording path reads live `GetAsyncKeyState`; this combined run's same test **passed under default parallel execution**, and the full combined suite passed 54 with 2 ignored. No serial/repeated rerun was necessary here, and none of these results establishes native keyboard/presentation qualification.

Command logs and artifact metadata are retained locally in `.tmp/release-1.0.0-candidate/` (`fmt.log`, `clippy.log`, `test-parallel.log`, `build-release.log`, `stress-release.log`, `artifacts.json`). The initial metadata probe encountered a PowerShell parser error and then unavailable `Get-FileHash`; the corrected .NET SHA256/VersionInfo probe succeeded without launching an executable.

## Export and preserved copies

Exact exported executable: [myosotis.exe](C:/Users/Aria/orca/workspaces/Myosotis/release-1.0.0-candidate/dist/Myosotis/myosotis.exe)

Export folder: [dist/Myosotis](C:/Users/Aria/orca/workspaces/Myosotis/release-1.0.0-candidate/dist/Myosotis)

Candidate SHA256: `d5ee58c2c865a3fb8783260b67efc66080e9acfc4d670275582d03fd8fea81fb`.

Candidate size: **686,080 bytes**. Cargo package version, Windows FileVersion, and Windows ProductVersion all remain **0.2.1** (numeric resource version `0,2,1,0`); product name **Myosotis**. This is a local pre-release review candidate for the proposed 1.0.0 work, **not a published 1.0.0**. No version/resource edits or downloadable release ZIP were made.

The release build, exported binary, and `.tmp/release-1.0.0-candidate/candidate/myosotis.exe` match SHA256. The candidate copy remains inside this worktree for later comparison. Baseline `.tmp/release-1.0.0-candidate/baseline/myosotis.exe` was copied read-only from #23's preserved baseline built at `47b6fd6`; **662,016 bytes**, version **0.2.1**, SHA256 `7c95fe33ec4c5a9f8f2e66433c81981e37109de0fd8f885ed575243d0881414d`. No file locking was encountered; neither binary was run or substituted for a running app.

## Native gaps and approval gate

[Manual acceptance checklist](manual-acceptance.md) is pending user execution. No desktop input/focus automation, visible app launch, clipboard replacement, smoke/theme/replay/preview/showcase scripts, or real stylus testing was authorized or run. Actual tray interactions and persistence through app restarts, real paste consumers/clipboard contention and ownership lifetime, visible lock fade and input latency, normal Quit immediately after Copy, redirected Pictures/error dialogs, and interactive baseline parity remain unqualified. Offscreen tests do not establish any flash root cause or disappearance; #23 is closed not planned, not fixed.

**User confirmation using this newly exported combined binary is required before master merge; WinGet remains blocked until the user confirms the requested issues/disposition.** Workspace/chat and preserved binaries are retained for direct inspection. No implementation blocker remains for preparing this local candidate; user/native confirmation is outstanding.
