# Coral annotations and periwinkle interface

Approved in the design discussion: use refined coral `#C74656` for every annotation and periwinkle `#5865D8` for Copy, selection borders, and resize handles. Preserve the app's existing icons exactly; preview icon drawings are not replacements. Keep neutral light/dark surfaces, toolbar geometry, guidance, shortcuts, and native tray styling.

Numbered marks use Segoe UI Semibold (Segoe UI family, semibold weight), 14 DIP, white glyphs, and 24-DIP-high badges. Width grows to fit the visible numeral group with 8 DIP per side. Center the visible glyph bounds horizontally and vertically. Cache each number layout for the renderer lifetime, sharing the treatment between overlay and export. No bundled fonts or dependencies.

Alternatives considered: current red with blue controls; periwinkle for both annotations and controls; refined coral with existing blue controls. The user selected refined coral annotations with periwinkle controls after visual comparison. Correct numeral alignment is required, including 10 and 20.

Preserve pre-existing stroke experiments and incremental caching. No path, stroke-width, input, icon, or menu behavior changes. Interactive performance remains a release acceptance requirement; automated geometry and raster checks do not establish human-perceived latency.

Related: issue #4, experience tracker #5. Standalone previews are under `.tmp/annotation-colors/` and are not shipped product assets.
