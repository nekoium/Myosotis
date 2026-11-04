use super::*;

#[test]
fn numeral_pixels_are_centered_and_padded_at_common_display_scales() -> Result<()> {
    unsafe {
        let write: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)?;
        let factory: ID2D1Factory = D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)?;
        let surface = super::super::capture::Surface::new(640, 160)?;
        let target = factory.CreateDCRenderTarget(&properties())?;
        target.BindDC(
            surface.dc,
            &RECT {
                left: 0,
                top: 0,
                right: 640,
                bottom: 160,
            },
        )?;
        let target: ID2D1RenderTarget = target.cast()?;
        let ink = target.CreateSolidColorBrush(&ANNOTATION_INK, None)?;
        let white = target.CreateSolidColorBrush(&color(1., 1., 1., 1.), None)?;
        for scale in [1., 1.25, 1.5, 2.] {
            let format = NumberBadge::format(&write, scale)?;
            let mut previous_width = 0.;
            for number in [1, 4, 7, 10, 11, 20, 100, usize::MAX] {
                let badge = NumberBadge::new(&write, &format, number, scale)?;
                target.BeginDraw();
                target.Clear(Some(&color(0.14, 0.14, 0.15, 1.)));
                badge.draw(&target, Point::new(320., 80.), &ink, &white);
                target.EndDraw(None, None)?;
                let mut left = 640;
                let mut right = 0;
                let mut top = 160;
                let mut bottom = 0;
                let mut coral_pixels = 0;
                for (i, pixel) in surface.pixels().as_chunks::<4>().0.iter().enumerate() {
                    let x = i % 640;
                    let y = i / 640;
                    if pixel[0] > 220 && pixel[1] > 220 && pixel[2] > 220 {
                        left = left.min(x);
                        right = right.max(x);
                        top = top.min(y);
                        bottom = bottom.max(y);
                    }
                    if pixel[..3] == [86, 70, 199] {
                        coral_pixels += 1;
                    }
                }
                assert!(
                    left <= right && top <= bottom,
                    "missing number {number}, scale {scale}"
                );
                let center_x = (left + right + 1) as f32 / 2.;
                let center_y = (top + bottom + 1) as f32 / 2.;
                assert!(
                    (center_x - 320.).abs() <= 1.,
                    "{number} at {scale}: x={center_x}"
                );
                assert!(
                    (center_y - 80.).abs() <= 1.,
                    "{number} at {scale}: y={center_y}"
                );
                assert!(left as f32 >= 320. - badge.half_width + 7. * scale - 1.);
                assert!((right + 1) as f32 <= 320. + badge.half_width - 7. * scale + 1.);
                assert!(top as f32 > 80. - badge.radius);
                assert!(((bottom + 1) as f32) < 80. + badge.radius);
                assert!(coral_pixels > 100, "missing coral fill");
                if number == 100 {
                    previous_width = badge.half_width;
                } else if number == usize::MAX {
                    assert!(
                        badge.half_width > previous_width,
                        "large number did not widen"
                    );
                }
            }
        }
        Ok(())
    }
}
