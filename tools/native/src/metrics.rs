//! Replay geometry and diagnostics retain the original probe's integer truncation and .NET rounding.
pub fn rounded(value: f64) -> i32 {
    value.round_ties_even() as i32
}
pub fn curve(width: i32, height: i32, seconds: i32, index: i32) -> (i32, i32) {
    let count = 240 * seconds;
    let fraction = index as f64 / count as f64;
    let phase = std::f64::consts::PI * 10.0 * seconds as f64;
    (
        50 + rounded(((width - 100) * index) as f64 / count as f64),
        height / 4 + rounded(60.min(height / 7) as f64 * (fraction * phase).sin()),
    )
}
pub fn ink(pixel: u32) -> bool {
    (pixel >> 16) & 255 > 180 && (pixel >> 8) & 255 < 150 && pixel & 255 < 160
}
pub fn percentile(values: &mut [f64], p: f64) -> f64 {
    values.sort_by(f64::total_cmp);
    values[((p * values.len() as f64).ceil() as usize)
        .saturating_sub(1)
        .min(values.len() - 1)]
}
pub fn fidelity(pixels: &[u32], width: i32, height: i32, seconds: i32) -> (f64, f64, usize, usize) {
    let mut distances = Vec::new();
    let mut missing = 0;
    for i in 10..240 * seconds - 10 {
        let (x, y) = curve(width, height, seconds, i);
        let mut nearest = 21.0;
        for dy in -20..=20 {
            for dx in -20..=20 {
                let distance = ((dx * dx + dy * dy) as f64).sqrt();
                if distance >= nearest
                    || x + dx < 0
                    || x + dx >= width
                    || y + dy < 0
                    || y + dy >= height
                {
                    continue;
                }
                if ink(pixels[((y + dy) * width + x + dx) as usize]) {
                    nearest = distance;
                }
            }
        }
        distances.push(nearest);
        if nearest > 3.0 {
            missing += 1;
        }
    }
    let median = percentile(&mut distances, 0.5);
    let p95 = percentile(&mut distances, 0.95);
    (median, p95, missing, distances.len())
}
pub fn export_difference(a: &[u32], b: &[u32], width: i32, height: i32) -> (usize, usize, usize) {
    let (mut different, mut mask, mut missing) = (0, 0, 0);
    for y in 10..height - 10 {
        for x in 10..width - 10 {
            let at = (y * width + x) as usize;
            if a[at] != b[at] {
                different += 1;
            }
            if ink(a[at]) != ink(b[at]) {
                mask += 1;
                let nearby = (-1..=1).any(|dy| {
                    (-1..=1).any(|dx| {
                        ink(if ink(a[at]) {
                            b[((y + dy) * width + x + dx) as usize]
                        } else {
                            a[((y + dy) * width + x + dx) as usize]
                        })
                    })
                });
                if !nearby {
                    missing += 1;
                }
            }
        }
    }
    (different, mask, missing)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dotnet_rounding_and_nearest_rank() {
        assert_eq!((rounded(2.5), rounded(3.5), rounded(-2.5)), (2, 4, -2));
        assert_eq!(percentile(&mut [10.0, 1.0, 4.0, 2.0], 0.5), 2.0);
        assert_eq!(percentile(&mut [10.0, 1.0, 4.0, 2.0], 0.95), 10.0);
    }
    #[test]
    fn mask_tolerance_does_not_hide_missing_ink() {
        let mut a = vec![0xffffffff; 40 * 40];
        let mut b = a.clone();
        a[20 * 40 + 20] = 0xffc74656;
        b[20 * 40 + 21] = 0xffc74656;
        assert_eq!(export_difference(&a, &b, 40, 40), (2, 2, 0));
        b[20 * 40 + 21] = 0xffffffff;
        b[20 * 40 + 23] = 0xffc74656;
        assert_eq!(export_difference(&a, &b, 40, 40), (2, 2, 2));
    }
    #[test]
    fn replay_coordinates_match_managed_reference() {
        // Golden hash from the former C# Math.Round/Math.Sin formulas, over 136,107 points.
        let mut hash = 0xcbf29ce484222325_u64;
        for width in [700, 1400, 2560] {
            for height in [400, 850, 1600] {
                for seconds in [1, 2, 60] {
                    for i in 0..=240 * seconds {
                        let (x, y) = curve(width, height, seconds, i);
                        for value in [x, y] {
                            hash = (hash ^ value as u64).wrapping_mul(0x100000001b3);
                        }
                    }
                }
            }
        }
        assert_eq!(hash, 0x149b64139a76e992);
    }
    #[test]
    fn missing_curve_is_rejected() {
        assert_eq!(
            fidelity(&vec![0xffffffff; 700 * 400], 700, 400, 2),
            (21.0, 21.0, 460, 460)
        );
    }
}
