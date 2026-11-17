Confirmed interaction for #20, following direct user discussion:

- Shift is a press-once toggle for canvas lock; auto-repeat must not toggle repeatedly.
- Lock prevents accidental completion from mouse double-clicks, stylus double-taps, and repeated Space presses. Each tap remains normal drawing input.
- While locked, Copy and Cancel are available only through the quick toolbar buttons. Enter and configured Copy/Cancel shortcuts (Ctrl/Escape by default) are disabled until unlocked.
- Drawing, tools, Space drawing, undo/redo, clear, and selection resizing remain available.
- A black lock icon in the same vector style as the quick toolbar icons sits inside the captured field's top-right corner. It fades in when locked, stays visible, and fades out when unlocked. It is an overlay indicator, excluded from exported pixels.
- Each new capture starts unlocked; there is no persisted lock preference. Shift remains fixed/reserved and is not rebindable.

Implementation choices: use existing vector icon drawing and opacity tween, 160 ms fade-in / 100 ms fade-out, respect Windows client-area animation preference, and reuse the existing capture timer. Preserve immediate input, history, and clipboard-failure recovery. Related completion discussion: #2.

Tracking:
- [x] Confirm lock scope, toggle, indicator, explicit completion, and reset behavior.
- [ ] Implement lock and focused input/rendering tests.
- [ ] Update interaction documentation and link implementation.
- [ ] Run formatting, Clippy, unit tests, and offscreen stroke stress.
- [ ] Report native interaction qualification separately; no desktop automation is authorized.
