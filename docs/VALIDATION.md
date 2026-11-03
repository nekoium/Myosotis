# Windows MVP validation

Validated on Windows 11 Enterprise x64, OS 10.0.26300, using Rust 1.99.0 with the MSVC target. The interactive display was 2560 × 1600 at 200% scaling. This establishes one working Windows configuration, not broad Windows/hardware support.

## Artifact

`tools/Package.ps1` produced `dist/Myosotis-0.1.0-windows-x64.zip`:

| Measurement | Bytes |
| --- | ---: |
| Executable, including icon, manifest, and static C runtime | 324,608 |
| Complete extracted package, including README and licenses | 331,255 |
| ZIP download | 174,742 |

The script rejects downloads at or above 10,000,000 bytes and extracted contents at or above 20,000,000 bytes. SHA-256 for this package: `f9ebd00a5564919075e2912561ee59d9c856c052f76807080ea49ab2df8613ba`.

PE import inspection found only Windows-provided libraries: kernel32, user32, gdi32, shell32, ole32, oleaut32, ntdll, d2d1, dwrite, and Windows API-set DLLs. There is no imported Visual C++ redistributable DLL or bundled application runtime. Windows Imaging Component is activated through COM.

## Checks completed

- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test --locked` pass. Four model tests cover history branches, numbering, pen/shape paths, invalid starts, and resize coordinates.
- The native smoke check passes against the executable in the portable package. It exercises the registered Ctrl+Alt+S shortcut; screenshot regions and whole monitors; white regions and whole-monitor canvases; all four annotation tools; discrete and held Space input; auto-repeat suppression for a numbered action; undo/redo; hover-armed resizing; and cancellation without replacing clipboard data.
- Exported bitmap and PNG pixels verify image size, white backgrounds, annotations, opacity, and stable annotation coordinates after resizing. The toolbar is excluded from export.
- A separate process holds the clipboard open during a copy attempt. The overlay keeps the drawing; releasing the clipboard and retrying succeeds.
- A controlled blue window changes to orange after capture. The exported crop retains the original blue pixels, proving the frozen-screen behavior.
- The controlled window regains foreground focus after copying. The fixture processes its Windows activation messages before the assertion.
- Repeated invocation and cancellation complete, followed by normal resident-process exit and tray cleanup.
- An interactive blank-canvas sketch containing rectangles, an arrow, and numbered marks was copied from the packaged executable and visibly pasted into Microsoft Paint. The test Paint window was then closed without saving.

## Theme preference validation

The implementation for [issue #6](https://github.com/nekoium/Myosotis/issues/6) passed formatting, Clippy with warnings denied, and all 11 unit tests. The settings test covers a missing file without creating a folder, Dark/Light round trips, unknown and incorrectly typed values, preservation of unrelated TOML values, malformed TOML, and a failed temporary-file write that leaves previous settings intact.

`tools/Test-Themes.ps1` passed against the release executable with an isolated `LOCALAPPDATA` under the project. It checked native menu selection marks, no settings creation on default launch, persistence after restarting, live Light/Dark/Light toolbar pixels during a capture, preserved selection and annotation history, and exported white canvas/coral pixels. The existing native smoke suite also passed. The previously running resident app was closed only while idle and restored afterward.

The release executable measured 533,504 bytes, versus 331,264 bytes before theme support. An isolated package containing the same executable, README, licenses, and images as `tools/Package.ps1` measured 622,656 bytes extracted and 339,092 bytes zipped. Both size limits passed. Existing `dist/` files and public release assets were not replaced.

These checks do not establish visual quality across all monitors, accessibility themes, long-session performance, or physical input latency. Save-error reporting is covered by code inspection and settings failure tests; a native permission-error dialog was not exercised.

## Measurements and limits

The final smoke run logged 15 captures at **78–112 ms** from the beginning of acquisition to overlay setup and invalidation. Seven successful clipboard exports took **9–75 ms** from composition through PNG encoding and clipboard transfer. These are internal wall-clock timings; capture timing does not include the delay until the first presented frame or hotkey delivery, and no percentile claim is made from this small run.

The resident process reported **74.2 MiB working set and 271 handles** after the full integration run. A separate shorter interactive capture/annotation run reported about **41 MiB** after copying. Graphics and encoder caches can remain resident even after captured pixel buffers and render targets are released; these observations are not leak or long-session benchmarks.

Physical input-to-visible-stroke latency has not been instrumented. Mouse drawing was visually checked, but real stylus hardware, pressure, erasers, multiple monitors with mixed DPI or negative origins, HDR, protected content, secure desktops, device loss, Explorer restart recovery, screen readers, and extended sessions remain unvalidated. Pressure/erasers and individual toolbar accessibility controls are not implemented. macOS and Linux frontends do not exist yet.

No public release, code signing, installer, auto-update, or startup registration was performed. See [CONTRIBUTING.md](../CONTRIBUTING.md) for repeatable checks and packaging.
