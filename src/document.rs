//! Platform-independent annotation document. Coordinates are image pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}
impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    pub fn distance(self, other: Self) -> f32 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}
impl Rect {
    pub fn between(a: Point, b: Point) -> Self {
        Self {
            left: a.x.min(b.x),
            top: a.y.min(b.y),
            right: a.x.max(b.x),
            bottom: a.y.max(b.y),
        }
    }
    pub fn width(self) -> f32 {
        self.right - self.left
    }
    pub fn height(self) -> f32 {
        self.bottom - self.top
    }
    pub fn contains(self, p: Point) -> bool {
        p.x >= self.left && p.x <= self.right && p.y >= self.top && p.y <= self.bottom
    }
    pub fn rounded(self) -> Self {
        Self {
            left: self.left.floor(),
            top: self.top.floor(),
            right: self.right.ceil(),
            bottom: self.bottom.ceil(),
        }
    }
    pub fn clamp(self, bounds: Self) -> Self {
        Self {
            left: self.left.clamp(bounds.left, bounds.right),
            top: self.top.clamp(bounds.top, bounds.bottom),
            right: self.right.clamp(bounds.left, bounds.right),
            bottom: self.bottom.clamp(bounds.top, bounds.bottom),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tool {
    #[default]
    Pen,
    Arrow,
    Rectangle,
    Number,
}
impl Tool {
    pub const ALL: [Self; 4] = [Self::Pen, Self::Arrow, Self::Rectangle, Self::Number];
}

#[derive(Clone, Debug)]
pub struct Annotation {
    pub tool: Tool,
    pub points: Vec<Point>,
    pub number: usize,
    pub width: f32,
}

enum Undo {
    Draw,
    Reset(Vec<Annotation>),
}
enum Redo {
    Draw(Annotation),
    Reset,
}

#[derive(Default)]
pub struct Document {
    pub selection: Option<Rect>,
    pub blank: bool,
    pub tool: Tool,
    pub annotations: Vec<Annotation>,
    pub active: Option<Annotation>,
    undo: Vec<Undo>,
    redo: Vec<Redo>,
}
impl Document {
    pub fn select(&mut self, bounds: Rect, blank: bool) {
        self.selection = Some(bounds.rounded());
        self.blank = blank;
    }
    pub fn begin(&mut self, point: Point, scale: f32) {
        if self.active.is_some() || !self.selection.is_some_and(|r| r.contains(point)) {
            return;
        }
        let number = self
            .annotations
            .iter()
            .filter(|a| a.tool == Tool::Number)
            .count()
            + 1;
        self.active = Some(Annotation {
            tool: self.tool,
            points: vec![point],
            number,
            width: 3.0 * scale,
        });
    }
    pub fn update(&mut self, point: Point) {
        if let Some(a) = &mut self.active {
            if a.tool == Tool::Pen {
                if a.points.last().unwrap().distance(point) >= 0.5 {
                    a.points.push(point);
                }
            } else if a.tool != Tool::Number {
                if a.points.len() == 1 {
                    a.points.push(point);
                } else {
                    a.points[1] = point;
                }
            }
        }
    }
    pub fn finish(&mut self) {
        if let Some(a) = self.active.take() {
            self.annotations.push(a);
            self.undo.push(Undo::Draw);
            self.redo.clear();
        }
    }
    pub fn reset(&mut self) {
        self.finish();
        if !self.annotations.is_empty() {
            self.undo
                .push(Undo::Reset(std::mem::take(&mut self.annotations)));
            self.redo.clear();
        }
    }
    pub fn undo(&mut self) {
        self.active = None;
        match self.undo.pop() {
            Some(Undo::Draw) => self.redo.push(Redo::Draw(self.annotations.pop().unwrap())),
            Some(Undo::Reset(annotations)) => {
                self.annotations = annotations;
                self.redo.push(Redo::Reset);
            }
            None => {}
        }
    }
    pub fn redo(&mut self) {
        if self.active.is_none() {
            match self.redo.pop() {
                Some(Redo::Draw(annotation)) => {
                    self.annotations.push(annotation);
                    self.undo.push(Undo::Draw);
                }
                Some(Redo::Reset) => {
                    self.undo
                        .push(Undo::Reset(std::mem::take(&mut self.annotations)));
                }
                None => {}
            }
        }
    }
}

/// Bit mask: left, top, right, bottom. Corners combine two edges.
pub fn hit_edge(r: Rect, p: Point, tolerance: f32) -> u8 {
    if !(Rect {
        left: r.left - tolerance,
        top: r.top - tolerance,
        right: r.right + tolerance,
        bottom: r.bottom + tolerance,
    })
    .contains(p)
    {
        return 0;
    }
    let mut edge = 0;
    if (p.x - r.left).abs() <= tolerance {
        edge |= 1;
    }
    if (p.y - r.top).abs() <= tolerance {
        edge |= 2;
    }
    if (p.x - r.right).abs() <= tolerance {
        edge |= 4;
    }
    if (p.y - r.bottom).abs() <= tolerance {
        edge |= 8;
    }
    edge
}

pub fn resize(r: Rect, edge: u8, p: Point, bounds: Rect, minimum: f32) -> Rect {
    let mut result = r;
    if edge & 1 != 0 {
        result.left = p.x.clamp(bounds.left, r.right - minimum);
    }
    if edge & 2 != 0 {
        result.top = p.y.clamp(bounds.top, r.bottom - minimum);
    }
    if edge & 4 != 0 {
        result.right = p.x.clamp(r.left + minimum, bounds.right);
    }
    if edge & 8 != 0 {
        result.bottom = p.y.clamp(r.top + minimum, bounds.bottom);
    }
    result.rounded()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn p(x: f32, y: f32) -> Point {
        Point::new(x, y)
    }
    #[test]
    fn history_branches_and_numbers_remain_sequential() {
        let mut d = Document::default();
        d.select(Rect::between(p(0., 0.), p(100., 100.)), true);
        d.tool = Tool::Number;
        for _ in 0..3 {
            d.begin(p(20., 20.), 1.);
            d.finish();
        }
        d.undo();
        d.undo();
        d.redo();
        assert_eq!(d.annotations.len(), 2);
        d.begin(p(30., 30.), 1.);
        d.finish();
        d.redo();
        assert_eq!(d.annotations.len(), 3);
        assert_eq!(d.annotations[2].number, 3);
    }
    #[test]
    fn resize_keeps_annotations_in_image_coordinates() {
        let mut d = Document::default();
        let bounds = Rect::between(p(0., 0.), p(200., 200.));
        d.select(Rect::between(p(10., 10.), p(100., 100.)), false);
        d.begin(p(20., 20.), 1.);
        d.update(p(40., 40.));
        d.finish();
        d.selection = Some(resize(
            d.selection.unwrap(),
            1 | 2,
            p(-10., 50.),
            bounds,
            1.,
        ));
        assert_eq!(d.selection.unwrap().left, 0.);
        assert_eq!(d.selection.unwrap().top, 50.);
        assert_eq!(d.annotations[0].points[0], p(20., 20.));
        assert_eq!(hit_edge(d.selection.unwrap(), p(0., 50.), 4.), 3);
    }
    #[test]
    fn invalid_starts_and_cancel_do_not_change_history() {
        let mut d = Document::default();
        d.begin(p(1., 1.), 1.);
        d.finish();
        assert!(d.annotations.is_empty());
        d.select(Rect::between(p(10., 10.), p(100., 100.)), false);
        d.begin(p(0., 0.), 1.);
        d.finish();
        assert!(d.annotations.is_empty());
        d.begin(p(20., 20.), 1.);
        d.active = None;
        assert!(d.annotations.is_empty());
    }
    #[test]
    fn reset_is_undoable_and_preserves_canvas_and_tool() {
        let mut d = Document::default();
        let selection = Rect::between(p(0., 0.), p(100., 100.));
        d.select(selection, true);
        d.tool = Tool::Number;
        for _ in 0..2 {
            d.begin(p(20., 20.), 1.);
            d.finish();
        }
        d.reset();
        assert!(d.annotations.is_empty());
        assert_eq!(d.selection, Some(selection));
        assert!(d.blank);
        assert_eq!(d.tool, Tool::Number);
        d.undo();
        assert_eq!(d.annotations.len(), 2);
        d.undo();
        assert_eq!(d.annotations.len(), 1);
        d.redo();
        d.redo();
        assert!(d.annotations.is_empty());
        d.undo();
        d.begin(p(30., 30.), 1.);
        d.finish();
        d.redo();
        assert_eq!(d.annotations.len(), 3);
        assert_eq!(d.annotations[2].number, 3);
    }
    #[test]
    fn empty_reset_keeps_redo_and_active_reset_keeps_stroke() {
        let mut d = Document::default();
        d.select(Rect::between(p(0., 0.), p(100., 100.)), false);
        d.begin(p(20., 20.), 1.);
        d.reset();
        d.undo();
        assert_eq!(d.annotations.len(), 1);
        d.undo();
        d.reset();
        d.redo();
        assert_eq!(d.annotations.len(), 1);
    }
    #[test]
    fn arrow_has_only_two_points_and_pen_retains_path() {
        let mut d = Document::default();
        d.select(Rect::between(p(0., 0.), p(100., 100.)), false);
        d.tool = Tool::Arrow;
        d.begin(p(10., 10.), 1.);
        d.update(p(20., 20.));
        d.update(p(30., 30.));
        d.finish();
        assert_eq!(d.annotations[0].points.len(), 2);
        assert_eq!(d.annotations[0].points[1], p(30., 30.));
        d.tool = Tool::Pen;
        d.begin(p(10., 10.), 1.);
        d.update(p(20., 20.));
        d.update(p(30., 30.));
        d.finish();
        assert_eq!(d.annotations[1].points.len(), 3);
    }
}
