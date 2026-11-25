# Myosotis 1.0.1 validation

## Scope

Maintenance release based on `ca9af34`. Includes the README title alignment and guide-image sizing, shared nekoium Scoop bucket instructions in the portable package, release metadata, and matching distribution manifests. No application source or dependency versions changed from 1.0.0. The embedded assembly version, which still read 0.2.1.0, is corrected to 1.0.1.0.

## Checks

- `cargo fmt --check`: passed.
- `cargo clippy --locked --all-targets -- -D warnings`: passed.
- `cargo test --locked`: 54 passed, 0 failed, 2 intentionally ignored.
- `tools/Package.ps1`: passed, including the download and installed-size limits.
- ZIP contents: all 9 expected files present, including the executable, README, license notices, flower SVG, and four guide images. No settings, symbols, external DLLs, or internal working files included.
- Executable extracted from the actual ZIP: FileVersion and ProductVersion are 1.0.1; embedded assemblyIdentity is 1.0.1.0.
- ZIP SHA-256 matches the checksum file and both distribution manifests.
- `winget validate --manifest packaging/winget --disable-interactivity`: passed. These are prepared manifests; this does not establish community catalog availability.
- `git diff --check`: passed.

Package: `Myosotis-1.0.1-windows-x64.zip`.

Downloaded bytes: **429,240**. Installed bytes: **797,342**.

SHA-256: `60832e0794d897a40f2bbf23cf98f5d3a15651990edd102a9637dff2dce1e2c4`.

## Limits

No desktop input, focus, clipboard, or visible application checks were run. The user's desktop was not made available for exclusive testing, and this release changes no application behavior. The ignored clipboard lifecycle and optional stroke stress checks were not run. Existing native input, stylus, HDR, and multi-monitor qualification limits remain. The executable is unsigned.

Publication, clean-download verification, shared-bucket synchronization, and isolated Scoop installation results are recorded separately after publication.
