# Rust tool migration

Status: implemented and interactively qualified against the former C# probe on the current desktop. No pointer movement invalidated the runs.

## Scope and approach

Replace `tools/stroke-eval/Probe.cs` and `tools/release/Showcase.cs` with a separate Rust package under `tools/native`. Keep the existing PowerShell commands, diagnostic modes, CSV columns, replay frequency, image sizes, and validation guards. Build only the requested release tool through `tools/native/Build.ps1`.

A separate package keeps tool dependencies and executables outside the application build and portable package. The probe uses Win32 input, pointer injection, screen capture, and clipboard APIs through `windows-sys`. The compositor uses the same native GDI+ rendering backend as the former System.Drawing implementation. Reading the performance summary uses `serde_json`.

The managed Chart control was replaced by native GDI+ bars and labels with the same data and 0–150 ms axis. Its layout changes inside the original chart rectangle. Keeping .NET solely for charting would defeat the migration; changing the entire image backend would introduce unnecessary output differences.

The application source, application Cargo manifest, and application lockfile are not part of this migration. Existing inline C# helpers in other PowerShell scripts are outside this two-file migration.

## Validation results

Validation used the working tree based on `e87fee3`. Unrelated changes already present in the workspace were preserved. Application checks therefore describe that working tree, not a clean checkout or a newly qualified release.

| Check | Result |
| --- | --- |
| Application formatting and Clippy | Passed |
| Application unit/offscreen tests | 21 passed, one optional stress test ignored |
| Optional release offscreen stroke stress | Passed; test suite elapsed 0.36 seconds, excluding compilation |
| Tool formatting and Clippy, including examples | Passed |
| Tool unit tests | Four passed |
| Replay coordinate comparison | Exact golden hash match across 136,107 C# reference points: three widths, three heights, and 1-, 2-, and 60-second traces |
| Saved release stroke diagnostics | 41 fidelity rows and one preview/export comparison match the existing reports exactly |
| Live density comparison | Five alternating C#/Rust runs per density at 0, 25, 100, and 250 strokes; 120 observations per build and density |
| Live coverage and scheduling | Both implementations: zero missing curve positions at every density. Maximum observed injection lateness: C# 0.254 ms; Rust 0.343 ms |
| Live input-dispatch p95 | C#: 0.297–0.361 ms; Rust: 0.444–0.478 ms |
| Live paint p95 | C#: 10.067–10.240 ms; Rust: 11.007–11.051 ms; within the `max(1 ms, 10%)` comparison tolerance |
| Live pressure and export | Both: pressure widths 2/8/6 px; six mask-edge differences and zero differences beyond 1 px. Rust clipboard copy: 45.887 ms; C#: 48.292 ms |
| Live history and long-stroke modes | Both completed at 250 history strokes and 20,000 active samples without errors |
| Live sustained replay | Both implementations passed all 14,380 coverage positions over 60 seconds at 240 Hz. Maximum injection lateness: C# 0.257 ms; Rust 0.305 ms |
| Output compatibility | `frames.csv`, `input.csv`, `fidelity.csv`, `history-costs.csv`, and `active.csv` headers match exactly |
| Before/after image | Pixel-identical to original C# output |
| Component image | Pixel-identical to original C# output |
| Stress image | All pixels outside the chart match; 44,824 pixels differ inside the chart rectangle |
| Entry-point guards | Both PowerShell and native probe reject execution without interactive opt-in |
| Argument handling | Native probe rejects missing executable and invalid duration before desktop access |
| PowerShell entry points | Parser checks passed; showcase wrapper built and ran successfully |

The freshly generated C# reference images also matched the checked-in release images byte for byte. Existing release PNGs were retained as historical assets. The Rust showcase was inspected visually, including chart values and labels.

The replay test preserves .NET ties-to-even rounding and the original arithmetic order. Image diagnostics retain the coral-pixel predicate, nearest-rank percentiles, three-pixel coverage threshold, and one-pixel preview/export mask tolerance. The native probe also releases its injected mouse button when replay fails.

### Showcase timing

Five complete image-generation runs per implementation, using the same saved inputs:

| Implementation | Median | Warm range, excluding first run |
| --- | ---: | ---: |
| C# | 202.53 ms | 200.67–203.46 ms |
| Rust | 192.49 ms | 191.97–194.88 ms |

Rust measurements include executable launch; C# measurements exclude compilation and time calls inside the PowerShell process. Compilation is excluded from both. These small, single-machine samples show no observed generation slowdown, not a general performance guarantee. The native wrapper now requires a Rust build on first use; subsequent invocations use Cargo's incremental build check.

The live comparison used the exact frozen release executable SHA-256 `bcd983ec4f3515ec83fc3d346210c57a29bd9113d0aef11422b4ee88a56ea48e`. The runner alternated C# and Rust probe order across five runs. Each implementation produced 120 observations per density and 2,300 coverage samples at each density. Both passed pressure/export, history, 20,000-sample active-stroke, and 60-second sustained replay modes. The resident app was restored and clipboard content restored after testing.

The Rust `input_ms` p95 is 0.08–0.18 ms higher in these runs. Paint p95 is 0.77–0.98 ms higher, remaining within the comparison tolerance. Both remain around 10–11 ms. This is a small single-machine paired qualification, not a universal performance guarantee.

Saved fidelity and export comparisons completed together in approximately 0.31 seconds in the release verifier. This is offline analysis, not live input or application paint latency.

## Reproduce unattended checks

Run the separate tool package checks from the repository root:

```powershell
cargo fmt --manifest-path tools/native/Cargo.toml --check
cargo clippy --manifest-path tools/native/Cargo.toml --locked --all-targets -- -D warnings
cargo test --manifest-path tools/native/Cargo.toml --locked
```

With the original synthetic captures and release evaluation results available under `.tmp`, regenerate without replacing the checked-in images:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools/release/Build-Showcase.ps1 -OutputDirectory .tmp/rust-tools-validation/rust
cargo run --manifest-path tools/native/Cargo.toml --locked --release --example verify_saved -- .
```

The verifier uses release results under `.tmp/stroke-eval/results/release-020-*` and the checked-in PNGs as references. Those local test inputs are not part of a fresh clone; missing inputs are not a passing result.

## Qualification limits

The authorized desktop comparison passed without detected external pointer movement, lost focus, interrupted runs, coverage misses, pressure failures, or export ink mismatches. It qualifies this probe and frozen executable on this desktop. It does not establish input-to-photon latency, physical stylus hardware behavior, other display scaling, or performance on other machines. The observed Rust dispatch/paint overhead is modest and inside the comparison tolerance, but remains measurable.
