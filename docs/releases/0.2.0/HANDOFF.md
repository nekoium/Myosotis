# Myosotis 0.2.0 release handoff

Release title: **Myosotis 0.2.0 — More natural strokes**

Tag to create when publishing: `v0.2.0`. This preparation does not publish or tag a release.

## Download assets

Upload these files from `dist/` as release attachments:

- `Myosotis-0.2.0-windows-x64.zip`
- `Myosotis-0.2.0-windows-x64.zip.sha256`

ZIP SHA-256: `d155191ab074dd40518b55261febb0792cc693d3f1fdbb3f05929eca528c33fd`

Archive size: 360,758 bytes. Installed portable folder: 663,392 bytes. Both are below the repository's size budgets. No symbols, local evidence, settings, or temporary files belong in the package.

## Images to upload manually

Copy [RELEASE-NOTES.md](RELEASE-NOTES.md) into GitHub's release description. At each matching HTML comment, drag in the corresponding PNG or use GitHub's image picker. Wait for GitHub to finish uploading and insert its hosted image Markdown, then remove the comment. Do not paste local paths or relative image links into the release description.

1. [01-before-after.png](01-before-after.png), 1800 × 1400: matched native 0.1.2/0.2.0 scenes, with equally enlarged stroke and badge details.
2. [02-components.png](02-components.png), 1800 × 1600: native dark/light toolbar states, actual guidance, four annotation tools, and synthetic pen pressure output.
3. [03-stress-test.png](03-stress-test.png), 1800 × 1640: actual 250-stroke replay patterns, the fresh five-run paint-time comparison, and the packaged executable's 60-second replay result.

The PNGs are for the release description. They are separate from the portable application ZIP. The complete image set is also copied to `dist/Myosotis-0.2.0-release-kit/` for convenience.

## Before publishing

- Review the notes and three PNGs, including the known limitations.
- Use the release preparation commit as the release target; push that commit before creating the tag.
- Attach the ZIP and checksum, and upload the three images into the description manually.
- Publish `v0.2.0`, then make the prepared Scoop manifest available. Its download URL will work only after publication.

Source changes and assets are prepared locally. No GitHub issue comments, release publication, startup registration changes, or external messages are part of this handoff.

See [VALIDATION.md](VALIDATION.md) for exact evidence, provenance, reproduction commands, and test limitations.
