# Myosotis 1.0.1

A small maintenance release updates the portable download with the shared **nekoium** Scoop bucket instructions, README layout refinements, and consistent Windows version metadata. Capture and drawing behavior is unchanged from 1.0.0.

## Changes

- New Scoop installations use the shared **nekoium** bucket.
- The README's flower aligns with the title, and the Input schema guide appears at a smaller size.
- Windows version information matches 1.0.1 throughout the package, including the embedded application manifest.

## Install with Scoop

New Scoop installations now use the shared **nekoium** bucket:

```powershell
scoop bucket add nekoium https://github.com/nekoium/scoop-bucket
scoop install nekoium/myosotis
```

Already installed through the original **myosotis** bucket? Keep using it. Both buckets receive the same updates; you do not need to reinstall or change buckets.

## Download and update

**Windows 11 x64 · Portable · No additional runtime required**

Quit the older Myosotis from its notification-area menu, extract `Myosotis-1.0.1-windows-x64.zip`, and run `Myosotis/myosotis.exe`.

Using Scoop? Quit Myosotis, then run:

```powershell
scoop update myosotis
```

If you use **Start at login**, turn it off before moving or deleting the previous portable copy, then enable it from the new executable's location.
