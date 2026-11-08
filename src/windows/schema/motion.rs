use std::time::{Duration, Instant};

pub const OPEN: Duration = Duration::from_millis(160);
pub const CLOSE: Duration = Duration::from_millis(100);
pub const COLOR: Duration = Duration::from_millis(60);

#[derive(Clone, Copy)]
pub struct Tween<const N: usize> {
    from: [f32; N],
    to: [f32; N],
    started: Instant,
    duration: Duration,
}
impl<const N: usize> Tween<N> {
    pub fn new(value: [f32; N], now: Instant) -> Self {
        Self {
            from: value,
            to: value,
            started: now,
            duration: Duration::ZERO,
        }
    }
    pub fn sample(&self, now: Instant) -> [f32; N] {
        if !self.active(now) {
            return self.to;
        }
        let progress = ease(
            now.saturating_duration_since(self.started).as_secs_f32() / self.duration.as_secs_f32(),
        );
        std::array::from_fn(|i| self.from[i] + (self.to[i] - self.from[i]) * progress)
    }
    pub fn active(&self, now: Instant) -> bool {
        self.from != self.to && now.saturating_duration_since(self.started) < self.duration
    }
    pub fn retarget(&mut self, to: [f32; N], duration: Duration, now: Instant) {
        if self.to == to {
            return;
        }
        *self = Self {
            from: self.sample(now),
            to,
            started: now,
            duration,
        };
    }
    pub fn settle(&mut self) {
        self.from = self.to;
    }
}

// Match the browser study's cubic-bezier(.2, 0, 0, 1), solving x for elapsed time.
fn ease(progress: f32) -> f32 {
    if progress <= 0. {
        return 0.;
    }
    if progress >= 1. {
        return 1.;
    }
    let (mut low, mut high) = (0_f32, 1_f32);
    for _ in 0..16 {
        let t = (low + high) * 0.5;
        let x = 0.6 * t * (1. - t).powi(2) + t.powi(3);
        if x < progress {
            low = t;
        } else {
            high = t;
        }
    }
    let t = (low + high) * 0.5;
    t * t * (3. - 2. * t)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn timing_reversal_and_settle_preserve_current_value() {
        assert_eq!(
            (OPEN.as_millis(), CLOSE.as_millis(), COLOR.as_millis()),
            (160, 100, 60)
        );
        let now = Instant::now();
        let mut fade = Tween::new([0.], now);
        fade.retarget([1.], OPEN, now);
        assert_eq!(fade.sample(now), [0.]);
        let middle = now + OPEN / 2;
        let value = fade.sample(middle)[0];
        assert!(value > 0.85 && value < 0.9);
        fade.retarget([0.], CLOSE, middle);
        assert_eq!(fade.sample(middle), [value]);
        // Repeating the target must not extend a fade's deadline.
        fade.retarget([0.], CLOSE, middle + Duration::from_millis(50));
        assert_eq!(fade.sample(middle + CLOSE), [0.]);
        assert!(!fade.active(middle + CLOSE));
        fade.retarget([1.], OPEN, middle + CLOSE);
        fade.settle();
        assert_eq!(fade.sample(middle + CLOSE), [1.]);
        assert!(!fade.active(middle + CLOSE));
    }
    #[test]
    fn interrupted_color_keeps_all_channels_continuous() {
        let now = Instant::now();
        let mut color = Tween::new([0., 0., 1.], now);
        color.retarget([1., 0., 1.], COLOR, now);
        let at = now + Duration::from_millis(20);
        let current = color.sample(at);
        color.retarget([0., 1., 0.], COLOR, at);
        assert_eq!(color.sample(at), current);
        assert_eq!(color.sample(at + COLOR), [0., 1., 0.]);
    }
}
