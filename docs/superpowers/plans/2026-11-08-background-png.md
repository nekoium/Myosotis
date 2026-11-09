# Background PNG clipboard Implementation Plan

> **For agentic workers:** Execute in this existing sole-worker session under the user's explicit “Now go” instruction. Preserve the final user-test and shipping gate.

**Goal:** Remove content-dependent PNG compression from overlay completion while retaining CF_DIB and PNG clipboard output.

**Architecture:** A clipboard owner module publishes prepared DIB memory and delayed PNG, tracks one current asynchronous encoding job, and handles render/ownership messages outside App's RefCell borrow. Workers encode owned top-down BGRA bytes using thread-local COM/WIC objects; only the resident UI thread publishes clipboard formats. A 10 ms UI timer polls worker completion, avoiding notifications to destroyed/reused window handles.

**Tech Stack:** Existing Rust, Win32, windows 0.58, WIC, std threads and JoinHandle results.

**Spec:** docs/superpowers/specs/2026-11-08-background-png-design.md

## Global Constraints

- Preserve original/source diagnostic artifacts and both runnable releases.
- No new dependencies or settings. PNG plus CF_DIB remain available.
- Preserve newer clipboard contents and cancellation behavior.
- No commit/push/merge/closure before user testing and shipping approval.
- Run cursor/focus/clipboard checks only within explicitly authorized desktop testing.

## Review Focus

- A PNG request arriving before encoding completes must wait for the current job without deadlock.
- Old completion notifications after another copy must not replace newer clipboard contents.
- EmptyClipboard can reenter the owner window; clipboard state must avoid App borrow conflicts.
- Quit must materialize delayed PNG while the owner window still exists.
- Worker failure must retain usable CF_DIB and free thread/bitmap/COM resources.

### Task 1: Owned pixels and encoding

Files: src/windows/capture.rs; new src/windows/clipboard.rs; new src/windows/clipboard_tests.rs.

- [x] Move the PNG encoder to a function accepting owned top-down BGRA data, dimensions; initialize COM on each worker and uninitialize on success/failure.
- [x] Prepare CF_DIB and a worker snapshot before opening clipboard; keep GDI handles on the original thread and flush rendering before reading DIB bytes.
- [x] Verify exact PNG round-trip pixels, opaque alpha, bottom-up DIB layout, and failed/disconnected worker handling through memory-only tests.

### Task 2: Clipboard lifecycle and app integration

Files: src/windows/clipboard.rs, src/windows/mod.rs, src/windows/capture.rs, src/windows/render.rs, Cargo.toml.

- [x] Publish CF_DIB first; advertise delayed PNG and close the overlay only after publication succeeds.
- [x] Handle completion, WM_RENDERFORMAT, WM_RENDERALLFORMATS, and WM_DESTROYCLIPBOARD outside the App mutable borrow. Guard proactive publication with current ownership; WM_DESTROYCLIPBOARD clears the current job, even on same-owner replacement, and workers never post stale results.
- [x] Render pending formats before owner window destruction; retain bitmap on PNG errors and the overlay on initial publication errors.
- [x] Preserve diagnostic source snapshots; remove diagnostic env switches and timing hooks from product source, and restore baseline presentation options.
- [x] Test stale completion rejection and immediate pending-result retrieval without real clipboard interaction.

### Task 3: Validation and comparison

Files: project-local .tmp/issue-16 scripts/artifacts; DESIGN.md; delegations/milestone-1.0.0/issue-16-result.md.

- [x] Run formatting, Clippy, locked unit tests, release build, whitespace checks; fix failures before further qualification.
- [ ] Build candidate separately and compare original/candidate using shortcut and direct double-click over maximized window versus desktop. Candidate built; comparison blocked by another issue's running resident, left untouched.
- [x] Qualify immediate CF_DIB and delayed PNG, replacement during encoding, consecutive copies, and shutdown in explicitly authorized synthetic native clipboard tests. Real destination-app paste and final visible completion comparison remain pending.
- [x] Update agreed behavior in DESIGN.md and durable results, keep source/artifacts reviewable, mark exact workspace in-review with unread marker.
