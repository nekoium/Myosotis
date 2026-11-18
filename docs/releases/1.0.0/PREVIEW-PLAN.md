# Local 1.0.0 release preview

Approved direction: three feature stories (canvas lock, optional local PNG saving, background PNG encoding), followed by interface refinements, update instructions, and limitations. Compare against v0.2.1. Use the restrained 0.2.0 showcase palette inside GitHub's light release-page layout.

Implementation plan:
- [x] Save the release-notes draft in RELEASE-NOTES.md.
- [x] Build a dependency-free preview.html with inline illustrated storyboards and existing native screenshots. Clearly distinguish illustrations from new capture evidence.
- [x] Check relative resources and render the page headlessly at desktop and phone widths.

Validation: headless Chromium inspection at 1440px and 390px completed with zero failures. Read both full-page contact sheets and both first-viewport screenshots. One retained warning concerns the illustrated selection/menu borders; stronger outlines intentionally identify application controls. Relative image and Markdown resources exist. `git diff --check` passed. No application tests or desktop interaction were needed.

Visual assessment: the GitHub-like shell and section hierarchy read clearly at both widths. Storyboards establish the release narrative, but are not final native showcase assets. The existing schema screenshot works as a reference; new old/new comparisons remain necessary. Saved preview-desktop.jpg, preview-overview.jpg, and preview-mobile.jpg for direct viewing. Temporary inspection output removed after retaining those previews.

Constraints: local preview only; no desktop automation, version changes, publication, tags, catalog submission, merge, or push. Actual package version remains 0.2.1. New native before/after captures and completion timings are not available. Do not claim a flash fix. No release assets exist yet; the preview's assets list must say so.

User revisions: remove Good to know from both release notes and preview; rename the completion section to “Capture before you even realize”; align tray labels in a shared column with a separate checkmark gutter. The historical screenshot substitution was rejected: restore the original before/after storyboard and its content, while making the illustrated app interface faithful to the actual app. Added canvas-lock-illustration.svg using source vector icon geometry, six 40-DIP toolbar cells, actual lock placement, selection handles, coral ink, and locked guidance. Keep the illustration explicitly labeled. Release title is now “Myosotis 1.0.0 — Capture it before you even realize”. Desktop/mobile renders inspected and saved again; zero inspection failures. Sidebar and interface-border warnings are intentional GitHub/application conventions.

Current priority: release notes only; README design belongs to a separate session. The user subsequently approved these visuals, requested inclusion of the new README commit, and explicitly authorized publishing 1.0.0. Release preparation includes `f14a112`, version/resource updates, the release download reference in README, packaging, tag/push and GitHub publication. Scoop is updated after verifying the published ZIP; WinGet is handled separately after handoff. No desktop automation is authorized.

Visual review: GitHub conventions intentionally take precedence over marketing-page design rules. System fonts, white/#f6f8fa surfaces, #1f2328 text, #0969da links; showcase coral #c74656 and periwinkle #5865d8. No invented measurements. Existing screenshot is historical and labeled accordingly.
