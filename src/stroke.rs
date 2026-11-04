//! Width profiles for mouse taper and stylus pressure. Captured positions are never filtered or resampled.
use crate::document::Point;

#[derive(Clone, Copy, Debug)]
pub struct Node {
    pub point: Point,
    pub radius: f32,
    pub distance: f32,
}

#[derive(Clone, Debug)]
pub struct Stroke {
    pub nodes: Vec<Node>,
    width: f32,
    time: u32,
    speed: f32,
}

impl Stroke {
    pub fn new(point: Point, width: f32, pressure: Option<f32>, time: u32) -> Self {
        Self {
            nodes: vec![Node {
                point,
                radius: Self::pressure_radius(width, pressure),
                distance: 0.,
            }],
            width,
            time,
            speed: 0.,
        }
    }
    fn pressure_radius(width: f32, pressure: Option<f32>) -> f32 {
        width
            * 0.5
            * pressure
                .filter(|p| p.is_finite())
                .map_or(1., |p| 0.35 + 1.3 * p.clamp(0., 1.))
    }
    pub fn push(&mut self, point: Point, pressure: Option<f32>, time: u32) {
        let last = *self.nodes.last().unwrap();
        let distance = last.point.distance(point);
        let elapsed = time.wrapping_sub(self.time).clamp(1, 100) as f32;
        self.time = time;
        let blend = 1. - (-elapsed / 24.).exp();
        self.speed += (distance / elapsed / self.width - self.speed) * blend;
        let target = if pressure.is_some_and(f32::is_finite) {
            Self::pressure_radius(self.width, pressure)
        } else {
            let response = if cfg!(feature = "stroke-natural") {
                0.22
            } else {
                0.
            };
            self.width * 0.5 * (1. + response * (0.4 - self.speed.min(1.4)))
        };
        let radius = last.radius + (target - last.radius) * blend;
        if distance == 0. {
            // Retain stationary pressure changes without a zero-length segment.
            self.nodes.last_mut().unwrap().radius = radius;
        } else {
            self.nodes.push(Node {
                point,
                radius,
                distance: last.distance + distance,
            });
        }
    }
    pub fn tail_length(&self) -> f32 {
        self.width * 4.
    }
    pub fn stable_segments(&self) -> usize {
        let cutoff = self.nodes.last().unwrap().distance - self.tail_length();
        self.nodes
            .partition_point(|n| n.distance <= cutoff)
            .saturating_sub(1)
    }
    pub fn disk(&self, index: usize, taper_end: bool) -> Node {
        self.shade(self.nodes[index], taper_end)
    }
    pub fn segment(&self, index: usize, taper_end: bool) -> Vec<Node> {
        let a = self.nodes[index];
        let b = self.nodes[index + 1];
        let length = self.nodes.last().unwrap().distance;
        let mut distances = vec![a.distance, b.distance];
        // Split only at width-profile landmarks, never at a pixel-relative sampling rate.
        for t in [0.25, 0.5, 0.75, 1.] {
            for distance in [self.width * 2.5 * t, length - self.tail_length() * t] {
                if distance > a.distance
                    && distance < b.distance
                    && (taper_end || distance <= self.width * 2.5)
                {
                    distances.push(distance);
                }
            }
        }
        distances.sort_by(f32::total_cmp);
        distances.dedup();
        distances
            .into_iter()
            .map(|distance| {
                let t = (distance - a.distance) / (b.distance - a.distance);
                self.shade(
                    Node {
                        point: Point::new(
                            a.point.x + (b.point.x - a.point.x) * t,
                            a.point.y + (b.point.y - a.point.y) * t,
                        ),
                        radius: a.radius + (b.radius - a.radius) * t,
                        distance,
                    },
                    taper_end,
                )
            })
            .collect()
    }
    fn shade(&self, mut node: Node, taper_end: bool) -> Node {
        if self.nodes.len() == 1 {
            return node;
        }
        let ramp = |distance: f32, length: f32| {
            let t = (distance / length).clamp(0., 1.);
            0.38 + 0.62 * t * t * (3. - 2. * t)
        };
        let start = ramp(node.distance, self.width * 2.5);
        let end = if taper_end {
            ramp(
                self.nodes.last().unwrap().distance - node.distance,
                self.tail_length(),
            )
        } else {
            1.
        };
        node.radius = (node.radius * start.min(end)).max(self.width * 0.16);
        node
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn positions_corners_tiny_moves_and_wrap_are_preserved() {
        let mut s = Stroke::new(Point::new(0., 0.), 3., None, u32::MAX - 3);
        let points = [
            Point::new(0.01, 0.),
            Point::new(10., 0.),
            Point::new(10., 10.),
            Point::new(0., 0.),
        ];
        for (i, p) in points.iter().enumerate() {
            s.push(*p, None, i as u32);
        }
        assert_eq!(s.nodes.len(), 5);
        for (i, p) in points.iter().enumerate() {
            assert_eq!(s.nodes[i + 1].point, *p);
        }
        assert!(s.nodes.iter().all(|n| n.radius.is_finite()));
    }
    #[test]
    fn committed_widths_never_change_as_stroke_grows() {
        let mut s = Stroke::new(Point::default(), 3., None, 0);
        for i in 1..100 {
            s.push(Point::new(i as f32, 0.), None, i * 5);
        }
        let committed = s.stable_segments();
        let radii: Vec<_> = (0..=committed).map(|i| s.disk(i, true).radius).collect();
        s.push(Point::new(200., 100.), None, 600);
        for (i, radius) in radii.iter().enumerate() {
            assert_eq!(*radius, s.disk(i, true).radius);
        }
    }
    #[test]
    fn a_two_sample_flick_has_a_full_body_without_dense_resampling() {
        let mut s = Stroke::new(Point::default(), 3., None, 0);
        s.push(Point::new(1000., 0.), None, 10);
        let segment = s.segment(0, true);
        assert_eq!(s.nodes.len(), 2);
        assert!(segment.len() <= 10);
        assert!(segment.iter().any(|n| n.radius > 1.4));
        assert!(segment.first().unwrap().radius < 0.7);
        assert!(segment.last().unwrap().radius < 0.7);
        assert!(segment.iter().all(|n| n.point.y == 0.));
    }
    #[test]
    fn mouse_speed_changes_width_only_in_natural() {
        let mut fast = Stroke::new(Point::default(), 3., None, 0);
        let mut slow = fast.clone();
        for i in 1..=100 {
            let point = Point::new(i as f32 * 6., 0.);
            fast.push(point, None, i);
            slow.push(point, None, i * 20);
        }
        assert_eq!(fast.nodes[50].point, slow.nodes[50].point);
        if cfg!(feature = "stroke-natural") {
            assert!(fast.disk(50, true).radius < 1.5);
            assert!(slow.disk(50, true).radius > 1.5);
        } else {
            assert_eq!(fast.disk(50, true).radius, 1.5);
            assert_eq!(slow.disk(50, true).radius, 1.5);
        }
    }
    #[test]
    fn pressure_overrides_speed_and_short_marks_remain_visible() {
        let mut fast = Stroke::new(Point::default(), 3., Some(0.8), 0);
        let mut slow = fast.clone();
        for i in 1..=100 {
            fast.push(Point::new(i as f32, 0.), Some(0.8), i);
            slow.push(Point::new(i as f32, 0.), Some(0.8), i * 20);
        }
        assert_eq!(fast.disk(50, true).radius, slow.disk(50, true).radius);
        assert!(fast.disk(50, true).radius > fast.disk(0, true).radius);
        let mut tiny = Stroke::new(Point::default(), 3., None, 0);
        tiny.push(Point::new(0.1, 0.), None, 1);
        assert!(tiny.disk(1, true).radius >= 0.48);
    }
    #[test]
    fn pressure_zero_absent_and_stationary_are_distinct() {
        assert!(Stroke::pressure_radius(3., Some(0.)) < Stroke::pressure_radius(3., None));
        assert!(Stroke::pressure_radius(3., Some(1.)) > Stroke::pressure_radius(3., None));
        assert_eq!(Stroke::pressure_radius(3., Some(f32::NAN)), 1.5);
        let mut s = Stroke::new(Point::default(), 3., Some(0.), 0);
        for t in 1..100 {
            s.push(Point::default(), Some(1.), t);
        }
        assert_eq!(s.nodes.len(), 1);
        assert!(s.nodes[0].radius > 2.);
        assert_eq!(s.disk(0, true).radius, s.nodes[0].radius);
    }
}
