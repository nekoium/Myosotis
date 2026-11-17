# Myosotis 1.0.0 — issue #20

Task: task_d92db9d29b08; dispatch: ctx_946d8f7f5b6a.
Coordinator: orca_session_id:pi_b8bf085b_6e5c_4458_8ac7_00b2b943b0f7.
Worker: orca_session_id:8fb92b30-2ced-4d62-b91c-ab9c62c03e73.

Implement https://github.com/nekoium/Myosotis/issues/20 (Hotkey to lock canvas) for 1.0.0. Desired scenario: rapid drawing without accidentally double-clicking and completing/closing capture. Current description is only “To avoid mis-double click and end while drawing”; locking semantics and shortcut are not agreed.

Read AGENTS.md, CONTRIBUTING.md, DESIGN.md, #20, related #2, and input/completion/settings/schema code before the first focused product question directly to the user. Discuss what lock prevents, toggle and indication, explicit copy/cancel, and persistence. Do not invent accepted choices. Preserve immediate drawing, history, keyboard drawing, rebinding, and clipboard recovery.

Use only this isolated Orca worktree based on origin/master 47b6fd6 or newer. Leave coordinator's old main checkout alone. Own canvas lock/input/completion, focused tests, and issue-specific docs. Other workers own #23 flash, #25 local save, #1 distribution. Shared input/schema/settings/mod.rs and DESIGN.md may overlap; report integration notes/dependencies. Never overwrite another branch. No release version changes, merges to master, publication, helper workers, or unsolicited redesign.

Record approved decisions/checklists in #20 and link implementation; update DESIGN.md for agreed interactions. Dispatch also forbids posting to GitHub during this run: prepare issue text locally and coordinate its publication rather than posting without clarification.

Run relevant formatting, Clippy, and unit/offscreen checks. Desktop input/focus/clipboard automation requires explicit availability for the whole run. Preserve runnable baseline/candidate if native comparison is needed. Acceptance: agreed lock prevents accidental double-click completion, preserves deliberate exit/copy and existing input, focused checks pass. Separate automated evidence from real interaction confirmation.

Write result.md alongside this brief with outcome, decisions, implementation/commits, actual checks, limitations, integration notes. Ask coordinator cross-worker questions via orchestration; product questions directly to user. End first turn after inspection with one focused user question; kickoff is not completion. Send worker_done exactly once when done and remain available for direct conversation.
