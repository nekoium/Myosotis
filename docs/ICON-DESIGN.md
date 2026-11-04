# Application icon

Selected direction for [issue #8](https://github.com/nekoium/Myosotis/issues/8): **A**, the tonal flower baseline. Five overlapping blue petals remain individually visible through flat color boundaries around a yellow center. No outlines, gradients, or shadows are used.

The softer palette and larger-center alternatives were considered; A retains the selected original proportions and stronger petal definition.

## Source and geometry

`tools/New-Icon.ps1` is the source of truth. It uses Windows System.Drawing without downloaded assets or fonts. The design occupies a 256-unit canvas and is rendered at four times that resolution before downsampling.

Each petal uses the same closed path, translated to `(128, 128)`, uniformly scaled by `1.16`, and rotated by successive 72-degree steps. The center uses the same translation and scale. This enlarges the entire flower by 16% within the existing canvas while preserving A's proportions and leaving transparent clearance around the tips:

```text
M -13,-4
C -48,-28 -51,-62 -34,-87
C -13,-115 35,-103 41,-76
C 49,-48 28,-12 -13,-4
Z
```

Petal colors in drawing order are `#648DF0`, `#7EA3F8`, `#6B95F2`, `#86AAF7`, and `#739CF4`. The center is a 46-unit-diameter disk at `(128, 128)`, filled with `#F5C85C`.

## Generated assets

- `assets/myosotis.ico`: transparent PNG frames at 16, 32, 48, and 256 pixels, embedded by the existing Windows resource build.
- `assets/myosotis.svg`: the same paths, transforms, and palette as vector artwork for the README.
- `assets/myosotis.png`: the same 256-pixel frame for raster uses.

Regenerate all three assets with the icon command in [CONTRIBUTING.md](../CONTRIBUTING.md). Keep them synchronized with the generator. The README displays the SVG beside its heading using GitHub's supported `align="absmiddle"` attribute to avoid baseline alignment. GitHub's own repository-name header does not support a custom repository icon.
