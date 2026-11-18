# Myosotis 1.0.0 validation

## Scope and acceptance

Includes the changes on `origin/master` through `a6d99e7`, the user-approved README refresh `f14a112`, approved release notes and illustrated showcases, and the 1.0.0 Cargo/Windows resource versions. The user confirmed the combined application candidate before the earlier merge and explicitly authorized this release publication in the release-notes session. No application behavior was changed during release preparation.

## Release preparation checks

- `cargo fmt --check`: passed.
- `cargo clippy --locked --all-targets -- -D warnings`: passed.
- `cargo test --locked`: 54 passed, 0 failed, 2 intentionally ignored.
- `tools/Package.ps1`: release build passed and complete portable package stayed within size limits. The first invocation timed out after building; a completed invocation produced the final package.
- ZIP inspected using .NET: 10 entries, required executable, licenses, README, logo and guide images present; no PDBs, DLLs, settings or internal delegation files.
- Executable extracted from the actual ZIP: FileVersion and ProductVersion both `1.0.0`.
- `git diff --check`: passed.

Final package: `Myosotis-1.0.0-windows-x64.zip`.

Downloaded bytes: **429,294**. Installed bytes: **797,388**.

SHA-256: `fbbadedb14aba7940e9dfaeef52a85e250e64da648aac55f4a6e636042d925c6`.

The portable package includes the README flower SVG as well as all four existing guide images. Its checked-in manifest is updated only after the GitHub download is available.

## Showcase provenance

`01-canvas-lock.png` and `02-save-to-local.png` are approved illustrations exported headlessly from `preview.html`, not desktop captures. Toolbar and lock vector geometry follow `src/windows/render.rs`. Annotation width is 3 DIP: arrow stroke 2.4 DIP, round caps, wings up to 15 DIP at ±0.5 radians; rectangle stroke 1.8 DIP and radius 3 DIP. The simplified scene contains one rectangle and one arrow. Clipboard and saved output use the same SVG. Tray labels share a left edge outside a separate checkmark column.

`03-background-png.png` diagrams implementation order without a latency or speedup measurement. `04-refinements.png` includes the existing native Input schema screenshot from 0.2.1 documentation as an explicitly identified reference. PNGs include their provenance captions. Source SVGs and the local preview remain alongside the release notes.

Desktop and mobile preview rendering was inspected headlessly at 1440 and 390 CSS pixels. No desktop interaction, visible application launch, pointer/focus automation, or system clipboard replacement occurred. The pre-existing combined-candidate offscreen/stress evidence is in `delegations/release-1.0.0-candidate/result.md`; it was not repeated for metadata/documentation edits.

## Limits

No new native input, paste-consumer, physical stylus, HDR or broad multi-monitor qualification was performed during packaging. The executable is unsigned. No confirmed Input schema flash fix is claimed: #23 was closed not planned after unreliable reproduction. No WinGet availability is claimed; catalog submission is owned by the separate catalog session and follows actual GitHub publication.
