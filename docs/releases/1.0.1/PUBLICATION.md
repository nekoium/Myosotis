# Myosotis 1.0.1 publication

Published [Myosotis 1.0.1](https://github.com/nekoium/Myosotis/releases/tag/v1.0.1) on 2026-11-25 as the latest stable release. The release title has no tagline. Tag `v1.0.1` points to release commit `dd37ac5`.

## Published assets

- `Myosotis-1.0.1-windows-x64.zip`: 429,240 bytes.
- `Myosotis-1.0.1-windows-x64.zip.sha256`: 98 bytes.

A fresh GitHub download of both assets matched the local release artifacts byte for byte. The downloaded ZIP's SHA-256 is `60832e0794d897a40f2bbf23cf98f5d3a15651990edd102a9637dff2dce1e2c4`, matching GitHub's asset digest, checksum file, source Scoop manifest, shared Scoop manifest, and prepared WinGet installer manifest.

## Shared Scoop bucket

The [Sync manifests workflow](https://github.com/nekoium/scoop-bucket/actions/runs/45548490566) completed successfully. The shared `bucket/myosotis.json` contains version 1.0.1 and the verified download hash.

An isolated installation through Scoop's installed library functions resolved `nekoium/myosotis` from a fresh shared-bucket clone. Release download, SHA-256 verification, extraction, current-version junction, GUI shim creation, and shortcut creation passed. The installed executable reports 1.0.1, and installed manifest and installation records are present. The shortcut target was verified.

The test kept installation files under a temporary project folder, redirected the Start menu shortcut there, and suppressed persistent PATH edits. It did not launch the app or modify the user's existing Scoop installation. Normal system PATH and Start menu integration, running-app upgrades, and application input/clipboard behavior were not tested in this release session.

The original project-owned bucket also contains 1.0.1, so existing installations continue to update without migration. WinGet manifests were updated and validated locally; no catalog submission or availability is claimed.
