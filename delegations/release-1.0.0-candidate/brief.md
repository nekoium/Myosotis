# Combined local review candidate brief

Prepare a local candidate on `nekoium/release-1.0.0-candidate` from `origin/master` at `47b6fd6e3cc7794c1307f8acfacd17c17aadb91a`, integrating #20 (`03c579403c7ab9f4bbf3bb1b9a9062a272fb9ee9`) and #25 (`00aec458e1eb9c40c57de74112250cb077ae4103`). Transfer only the relevant #23 disposition sentence from its uncommitted DESIGN.md edit: closed not planned at the user's request after unreliable reproduction; no flash fix or confirmed cause.

Preserve Shift lock, toolbar-only Copy/Cancel exits, overlay-only fading indicator, local-save preference and exact shared PNG, timer ID 2, clipboard ownership/retry/shutdown, and existing package version. Review the combined paths; run fmt, Clippy, unit/offscreen checks, release build, and release offscreen stroke stress. Preserve baseline/candidate copies and export `dist/Myosotis/myosotis.exe` with source/version/hash evidence and a manual acceptance checklist.

Do not launch the app, drive desktop input/focus/clipboard, modify other worktrees, push, merge master, tag, publish, close issues, or submit WinGet. User confirmation with the newly exported combined binary is required before master merge and WinGet. This is a local review candidate, not a published 1.0.0 release.
