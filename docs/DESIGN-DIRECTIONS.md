# Interface direction decision

Status: the dark rectangular direction without a perimeter outline is selected.
Related problems: [annotation appearance #4](https://github.com/nekoium/Myosotis/issues/4), [control clarity #3](https://github.com/nekoium/Myosotis/issues/3), and [experience tracker #5](https://github.com/nekoium/Myosotis/issues/5).

## Selected design

Use an opaque charcoal surface (`#242426`), near-white vector icons and text, neutral selected and hover fills, and a blue Copy action. The toolbar and existing guidance surface have no perimeter outline or shadow. The compact rectangular form preserves the captured content while keeping common actions within reach.

Six 40-DIP cells replace the previous six 92-DIP text cells. The toolbar contains pen, arrow, rectangle, numbered mark, overlapping-page Copy, and close icons. Hovering shows the action and its shortcut in the existing guidance surface. Leaving the toolbar restores a short workflow hint. Error messages retain priority over contextual descriptions.

Annotation defaults use coral ink, rounded pen and arrow caps, finer arrows and rectangle outlines, and smaller numbered markers. Clipboard export uses the same annotation renderer. Keyboard shortcuts, completion gestures, background toggling, delayed border resizing, and toolbar placement keep their existing semantics.

## Alternatives considered

- Material: a dark capsule with 48-DIP cells, larger icons, and stronger tonal selection.
- Light macOS-inspired: a compact rectangle with 40-DIP cells and quiet neutral control states. Its light surfaces were harder to distinguish against light captured applications.
- Dark rectangle with an outline: the same controls and layout with a single 1-DIP gray perimeter around the toolbar and guidance surface. This separates the surfaces more clearly against dark content; the version without the outline was selected for its minimal appearance.

The selected direction retains the compact rectangle and uses a stable dark appearance across light and dark screenshots. There is no automatic theme sampling, shadow, blur, animation, new configuration, or additional runtime dependency. This is a macOS-inspired Windows interface, not a native macOS port.

## References

- [Material icon buttons](https://m3.material.io/components/icon-buttons/guidelines) and the [Material Web specification](https://github.com/material-components/material-web/blob/main/docs/components/icon-button.md): consistent icon geometry, selected states, and clear primary actions.
- [Material color roles](https://github.com/material-components/material-web/blob/main/docs/theming/color.md): separate surface, on-surface, primary, and container roles.
- [Apple toolbars](https://developer.apple.com/design/human-interface-guidelines/toolbars): familiar symbols, grouping, and restrained controls.
- [Apple interface icons](https://developer.apple.com/design/human-interface-guidelines/icons): simplified metaphors, consistent stroke weight, optical alignment, and vector scaling.

Icons are original geometric drawings; no SF Symbols, downloaded fonts, or catalog assets are bundled.

## Verification and limits

Both dark perimeter variants passed formatting, Clippy with warnings denied, all 10 unit tests, and the native smoke suite. The smoke suite covers invocation, cancellation, four tools, Space drawing, history/reset, resizing, double-click rollback, clipboard contention/retry, DIB/PNG output, background switching, frozen capture, foreground restoration, repeated invocation, and shutdown. Toolbar hit testing and smoke coordinates share the selected design's dimensions.

`tools/Preview-Design.ps1` creates a synthetic scene without exporting desktop content or changing the clipboard. It accepts `-Theme Light` (default) or `-Theme Dark`. Both candidates were visually inspected against both scenes at 200% scale.

The earlier Material and light macOS-inspired builds added 3,072 executable bytes over the baseline. Sequential smoke timing samples showed no demonstrated performance advantage for either direction. These measurements do not establish input-to-pixel latency or long-stroke frame times; complete package size was not remeasured.

Other DPI settings, mixed-DPI monitors, high contrast, and fullscreen overlap with undimmed application content still need visual review. The custom toolbar lacks a UI Automation provider and Tab navigation. Hover descriptions and direct shortcuts improve discoverability but do not provide full screen-reader accessibility. Related issues remain open for those outstanding outcomes.
