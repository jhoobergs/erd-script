#[derive(Debug, Clone)]
pub struct Grid(Vec<Circle>);

impl Grid {
    pub fn new() -> Grid {
        Grid(Vec::new())
    }
    pub fn add_circle(&mut self, radius: f64) {
        if self.0.is_empty() {
            self.0.push(Circle {
                center: Point { x: 0.0, y: 0.0 },
                radius,
            })
        } else {
            let mut new_circle = None;
            for circle in self.0.iter() {
                for (x_change, y_change) in vec![(1.0, 0.0), (0.0, 1.0), (-1.0, 0.0), (0.0, -1.0)] {
                    let possible_center = circle.center.moved(
                        x_change * (circle.radius + radius),
                        y_change * (circle.radius + radius),
                    );
                    let possible_circle = Circle {
                        center: possible_center,
                        radius,
                    };
                    if self.can_add_circle(&possible_circle) {
                        new_circle = Some(possible_circle);
                        break;
                    }
                }
                if new_circle.is_some() {
                    break;
                }
            }
            self.0.push(new_circle.expect("Circle position not found"))
        }
    }
    pub fn can_add_circle(&self, circle: &Circle) -> bool {
        for c in self.0.iter() {
            if c.overlaps_with(circle) {
                return false;
            }
        }
        return true;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Point {
    x: f64,
    y: f64,
}

impl Point {
    fn distance(&self, other: &Point) -> f64 {
        ((self.x - other.x).powf(2.0) + (self.y - other.y).powf(2.0)).sqrt()
    }
    fn moved(&self, x: f64, y: f64) -> Point {
        Point {
            x: x + self.x,
            y: y + self.y,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Circle {
    center: Point,
    radius: f64,
}

impl Circle {
    fn overlaps_with(&self, other: &Circle) -> bool {
        let distance = self.center.distance(&other.center);
        distance < self.radius + other.radius
    }
}

pub trait Draw {
    fn radius(&self) -> f64;
}

#[derive(Debug, Clone)]
pub struct TextElement {
    content: String,
}

impl TextElement {
    fn height() -> f64 {
        20.0 // TODO
    }

    fn width(&self) -> f64 {
        (self.content.len() * 10) as f64 // TODO
    }

    pub fn new(content: String) -> Self {
        Self { content }
    }
}

impl Draw for TextElement {
    fn radius(&self) -> f64 {
        let largest = if Self::height() > self.width() {
            Self::height()
        } else {
            self.width()
        };
        largest / 2.0
    }
}

#[derive(Debug, Clone)]
pub struct EntityElement {
    name: TextElement,
    attributes: Vec<AttributeElement>,
}

impl EntityElement {
    pub fn new(s: String) -> Self {
        Self {
            name: TextElement::new(s),
            attributes: Vec::new(), // TODO
        }
    }
}

impl Draw for EntityElement {
    fn radius(&self) -> f64 {
        self.name.radius()
    }
}

#[derive(Debug, Clone)]
pub struct AttributeElement {
    name: TextElement,
}

impl AttributeElement {
    pub fn new(s: String) -> AttributeElement {
        Self {
            name: TextElement::new(s),
        }
    }
}

impl Draw for AttributeElement {
    fn radius(&self) -> f64 {
        self.name.radius()
    }
}

#[derive(Debug, Clone)]
pub struct RelationElement {
    name: TextElement,
}

impl RelationElement {
    pub fn new(s: String) -> RelationElement {
        RelationElement {
            name: TextElement::new(s),
        }
    }
}

impl Draw for RelationElement {
    fn radius(&self) -> f64 {
        self.name.radius()
    }
}
