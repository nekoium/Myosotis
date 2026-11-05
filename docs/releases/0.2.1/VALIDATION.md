# Myosotis 0.2.1 release validation

## Scope

Release includes the changes committed since v0.2.0: customizable shortcuts, Input schema, tray-click access, and the CJK IME shortcut fix. Existing untracked delegation and promotional studies are excluded. The pre-existing capture.rs status entry has identical content to HEAD after CRLF normalization and is excluded from the release commit.

## Checks

- `cargo fmt --check`: passed.
- `cargo clippy --locked --all-targets -- -D warnings`: passed.
- `MYOSOTIS_SCHEMA_PREVIEW=1 cargo test --locked`: 28 passed, one optional diagnostic ignored.
- `winget validate --manifest packaging/winget`: passed. Community catalog submission remains separate.
- `tools/Package.ps1` with `CARGO_TARGET_DIR=.tmp/release-0.2.1-build`: passed. The normal release executable was locked by the resident app; the isolated output allowed packaging without disturbing it. Packaging now reads Cargo's target directory.
- Portable ZIP: 397,666 bytes downloaded; 729,986 bytes installed. Required executable, documentation, licenses, and schema image verified inside ZIP.
- Executable FileVersion: 0.2.1.
- SHA-256: `8814bbe7f38caa912979767937ec51cfe03d77f25caebb033116f1dc53dac740`. Matches Scoop and WinGet manifests.
- The 688 × 588 Input schema image comes from the current application's offscreen rendering test with default shortcuts. It was inspected visually and converted from BMP to PNG without resizing.

## Limits

No desktop-driving smoke, live tray/hotkey/clipboard tests, or Scoop install/upgrade were run in this release session. These require a separate available desktop or would change the user's active installation. Earlier Input schema checks are recorded in docs/INPUT-SCHEMA-VALIDATION.md; they do not replace current packaged native-integration qualification.

Rim candidate: release-0.2.1, n42acfab94f97aff.
