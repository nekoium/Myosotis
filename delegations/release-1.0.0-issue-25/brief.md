# Issue #25 — local image saving

Own issue: https://github.com/nekoium/Myosotis/issues/25, currently “Option to save to local”, body “In tray menu”.

## Scope and constraints

- Isolated Orca worktree `nekoium/release-1.0.0-issue-25`, verified HEAD `47b6fd6` (merged #24).
- Discuss product choices directly with user after inspecting existing code; no automatic saving or tray semantics assumed.
- Preserve capture/drawing speed, existing clipboard errors and delayed PNG ownership/shutdown handling.
- Keep encoding and file I/O outside drawing paths; preserve image bounds, annotations and background.
- Own local export/tray code, focused tests and issue-specific docs; coordinate overlaps in `src/windows/mod.rs` and `DESIGN.md`.
- Do not bump versions, publish, merge, delegate, or use visible desktop/clipboard automation without explicit availability.
- The initial dispatch prohibited GitHub posting; the user subsequently explicitly authorized posting the confirmed change as an issue reply. Posted only that reply: https://github.com/nekoium/Myosotis/issues/25#issuecomment-6952374179. Prepare further issue/checklist updates locally for the coordinator.
- #20 owns Lock/input/schema actions; notify coordinator before any save-related input/schema edits. #23 owns presentation lifecycle/motion. Integration order is provisionally #23, #20, #25; report exact shared-file edits and do not merge other branches.

## Inspection

Read AGENTS.md, CONTRIBUTING.md, DESIGN.md, README.md, settings, tray/completion handling, capture Surface, render export, background clipboard encoding and memory-only clipboard tests. Read issue #25 and listed all open issues / searched related issues. Current copy flattens current selection through Renderer::export, publishes CF_DIB + delayed PNG, closes and discards overlay, starts worker encoding; clipboard ownership can subsequently be replaced. No existing capture-file output exists. README currently explicitly promises captures are never saved to disk.

## Approved user decision

Persistent **Save to local** tray checkbox below **Start at login**, default off; each successful completed copy also saves PNG to Windows Pictures/Screenshots using local-time names such as `Screenshot 2026-11-15 215736.png`, collision-safe suffixes, worker encoding/I/O, folder creation as needed, and errors retaining clipboard output. Canceled captures do not save. Rename **Dark Theme** to **Dark theme**. User approved implementation and posting the decision reply, then requested commit and sync (push issue branch).

## Acceptance checklist

- [x] User approves concrete source/timing/destination workflow.
- [x] Correct selected bounds, background, annotations; no toolbar.
- [x] Cancel/failure preserves work and retry opportunity (cancel writes nothing; save failures preserve successful clipboard output).
- [x] Clipboard completion, ownership and shutdown retained; native clipboard qualification remains separate.
- [x] Privacy and interaction documentation match actual writes.
- [x] fmt, clippy, focused unit/offscreen checks pass; see result for intermittent existing schema test.
- [x] Native dialog interaction qualification reported honestly (no Save As dialog; native tray/error dialogs not exercised).
