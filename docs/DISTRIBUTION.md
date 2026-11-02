# Distribution

## GitHub Releases

The canonical Windows download is the versioned portable ZIP attached to a GitHub release. Never replace an already published ZIP: Scoop and WinGet verify its SHA-256. Publish fixes under a new version.

For a new version:

1. Update `Cargo.toml`, `assets/myosotis.rc`, and `assets/myosotis.manifest`. Refresh `Cargo.lock` with Cargo.
2. Update user documentation before building because README is included in the ZIP.
3. Run the checks in CONTRIBUTING.md and `tools/Package.ps1`. Run the native smoke check against the packaged executable on an idle desktop.
4. Update the Scoop manifest and WinGet manifests with the version, permanent release URL, and ZIP hash. Add release notes under `docs/releases/`.
5. Commit, push, create and push the matching `v<version>` tag, and publish the GitHub release with the ZIP and `.sha256` file.
6. Verify a clean download's checksum and an installation through Scoop.

## Scoop: project-owned bucket

`bucket/myosotis.json` makes this repository a Scoop bucket. No community submission or separate account is required:

```powershell
scoop bucket add myosotis https://github.com/nekoium/Myosotis
scoop install myosotis/myosotis
```

The manifest points to GitHub release assets and adds a command shim and Start menu shortcut. New releases require a version/hash update to this bucket. The `checkver` and `autoupdate` metadata help maintainers discover and generate those updates; they do not run automatically in this repository. Quit Myosotis before upgrading or uninstalling it.

Submission to Scoop Extras would be a separate pull request subject to its inclusion criteria and review. The custom bucket works without acceptance into Extras.

References: [Scoop manifests](https://github.com/ScoopInstaller/Scoop/wiki/App-Manifests), [custom buckets](https://github.com/ScoopInstaller/Scoop/wiki/Buckets).

## WinGet: community catalog submission

Prepared manifests are in `packaging/winget/`, with package ID `nekoium.Myosotis`. They describe the ZIP's nested portable executable and its `myosotis` command alias. Local validation:

```powershell
winget validate --manifest packaging/winget
```

To request public listing, fork `microsoft/winget-pkgs`, copy the three manifests into `manifests/n/nekoium/Myosotis/<version>/`, test install/uninstall in Windows Sandbox, and open a pull request. Microsoft runs automated checks and may require manual review. A validated local manifest does not mean the package is accepted or searchable. Once accepted, users can run:

```powershell
winget install --id nekoium.Myosotis --exact
```

There is no community submission in this release task. Do not advertise that command as working until the catalog actually lists the app. Code signing is separate from manifest submission; it can improve download trust, but approval remains the catalog maintainers' decision.

Reference: [Microsoft's submission process](https://learn.microsoft.com/en-us/windows/package-manager/package/repository).

## Chocolatey: separate package and moderation

Chocolatey Community Repository needs its own NuGet package: a `.nuspec` plus `tools/chocolateyInstall.ps1` and uninstall handling. The install script should download the canonical GitHub ZIP, verify SHA-256, and set up the portable executable/shortcut. Test install, upgrade, running-app behavior, and uninstall; build with `choco pack`; submit with `choco push` using a Chocolatey account's API key. New packages enter community moderation before general availability.

No Chocolatey package or account was configured for this release. GitHub credentials do not authorize Chocolatey publishing. The recommended initial distribution is GitHub Releases plus the project-owned Scoop bucket, with WinGet as the next catalog submission.

Reference: [Chocolatey package creation](https://docs.chocolatey.org/en-us/create/create-packages/).
