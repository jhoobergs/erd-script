use plotters::prelude::Circle as PlotCircle;
use plotters::prelude::*;

#[derive(Debug, Clone)]
pub struct Grid(Vec<(String, Circle)>);

impl Grid {
    pub fn new() -> Grid {
        Grid(Vec::new())
    }
    pub fn add_circle(&mut self, radius: isize, name: String) {
        if self.0.is_empty() {
            self.0.push((
                name,
                Circle {
                    center: Point { x: 0, y: 0 },
                    radius,
                },
            ))
        } else {
            let mut new_circle = None;
            for (_, circle) in self.0.iter() {
                for (x_change, y_change) in vec![(1, 0), (0, 1), (-1, 0), (0, -1)] {
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
            self.0
                .push((name, new_circle.expect("Circle position not found")))
        }
    }
    pub fn can_add_circle(&self, circle: &Circle) -> bool {
        for (_, c) in self.0.iter() {
            if c.overlaps_with(circle) {
                return false;
            }
        }
        return true;
    }
    // Moved grid so nothing is left or above coordinate 0
    pub fn normalized(&self) -> NormalizedGrid {
        if self.0.is_empty() {
            return NormalizedGrid(Vec::new());
        }
        let min_x = self
            .0
            .iter()
            .map(|(_, c)| c.center.x - c.radius)
            .min()
            .expect("Missing circles?");
        let min_y = self
            .0
            .iter()
            .map(|(_, c)| c.center.y - c.radius)
            .min()
            .expect("Missing circles?");
        NormalizedGrid(
            self.0
                .iter()
                .map(|(n, c)| (n.clone(), c.moved(-min_x, -min_y)))
                .collect(),
        )
    }
}

#[derive(Debug, Clone)]
pub struct NormalizedGrid(Vec<(String, Circle)>);

impl NormalizedGrid {
    pub fn draw(&self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        let max_x = self
            .0
            .iter()
            .map(|(_, c)| c.center.x + c.radius)
            .max()
            .expect("Missing circles") as u32;
        let max_y = self
            .0
            .iter()
            .map(|(_, c)| c.center.y + c.radius)
            .max()
            .expect("Missing circles") as u32;
        let root = BitMapBackend::new(path, (max_x, max_y)).into_drawing_area();
        root.fill(&WHITE)?;

        for (_, circle) in self.0.iter() {
            root.draw(&PlotCircle::new(
                (circle.center.x as i32, circle.center.y as i32),
                circle.radius as i32,
                Into::<ShapeStyle>::into(&BLACK),
            ))?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Point {
    x: isize,
    y: isize,
}

impl Point {
    fn distance(&self, other: &Point) -> f64 {
        (((self.x - other.x).pow(2) + (self.y - other.y).pow(2)) as f64).sqrt()
    }
    fn moved(&self, x: isize, y: isize) -> Point {
        Point {
            x: x + self.x,
            y: y + self.y,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Circle {
    center: Point,
    radius: usize,
}

impl Circle {
    fn overlaps_with(&self, other: &Circle) -> bool {
        let distance = self.center.distance(&other.center);
        distance < (self.radius + other.radius) as f64
    }
    fn moved(&self, x: isize, y: isize) -> Circle {
        Circle {
            center: self.center.moved(x, y),
            radius: self.radius,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Rectangle {
    center: Point,
    width: usize,
    height: usize,
}

impl Rectangle {
    /*fn overlaps_with(&self, other: &Ellipse) -> bool {
        let distance = self.center.distance(&other.center);
        distance < (self.radius + other.radius) as f64
    }*/
    fn moved(&self, x: isize, y: isize) -> Self {
        Self {
            center: self.center.moved(x, y),
            width: self.width,
            height: self.height,
        }
    }
}

#[derive(Debug, Clone, Copy)]

pub trait Draw {
    fn radius(&self) -> isize;
}

#[derive(Debug, Clone)]
pub struct TextElement {
    content: String,
}

impl TextElement {
    fn height() -> isize {
        20 // TODO
    }

    fn width(&self) -> isize {
        self.content.len() as isize * 10 // TODO
    }

    pub fn new(content: String) -> Self {
        Self { content }
    }
}

impl Draw for TextElement {
    fn radius(&self) -> isize {
        let largest = if Self::height() > self.width() {
            Self::height()
        } else {
            self.width()
        };
        largest / 2
    }
}

#[derive(Debug, Clone)]
pub struct EntityElement {
    name: TextElement,
    attributes: Vec<AttributeElement>,
}

impl EntityElement {
    pub fn new(s: String, attributes: Vec<String>) -> Self {
        Self {
            name: TextElement::new(s),
            attributes: attributes
                .into_iter()
                .map(|a| AttributeElement::new(a))
                .collect(),
        }
    }
}

impl Draw for EntityElement {
    fn radius(&self) -> isize {
        self.name.radius()
            + self
                .attributes
                .iter()
                .map(|a| a.radius())
                .max()
                .unwrap_or(0)
    }
}

#[derive(Debug, Clone)]
pub struct EntityGrid((String, Rectangle), Vec<(String, Ellipse)>);

impl EntityGrid {
    pub fn new(name: String, rect: Rectangle) -> Grid {
        EntityGrid((name, rect), Vec::new())
    }
    pub fn add_circle(&mut self, radius: isize, name: String) {
        if self.0.is_empty() {
            self.0.push((
                name,
                Circle {
                    center: Point { x: 0, y: 0 },
                    radius,
                },
            ))
        } else {
            let mut new_circle = None;
            for (_, circle) in self.0.iter() {
                for (x_change, y_change) in vec![(1, 0), (0, 1), (-1, 0), (0, -1)] {
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
            self.0
                .push((name, new_circle.expect("Circle position not found")))
        }
    }
    pub fn can_add_circle(&self, circle: &Circle) -> bool {
        for (_, c) in self.0.iter() {
            if c.overlaps_with(circle) {
                return false;
            }
        }
        return true;
    }
    // Moved grid so nothing is left or above coordinate 0
    pub fn normalized(&self) -> NormalizedGrid {
        if self.0.is_empty() {
            return NormalizedGrid(Vec::new());
        }
        let min_x = self
            .0
            .iter()
            .map(|(_, c)| c.center.x - c.radius)
            .min()
            .expect("Missing circles?");
        let min_y = self
            .0
            .iter()
            .map(|(_, c)| c.center.y - c.radius)
            .min()
            .expect("Missing circles?");
        NormalizedGrid(
            self.0
                .iter()
                .map(|(n, c)| (n.clone(), c.moved(-min_x, -min_y)))
                .collect(),
        )
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
    fn radius(&self) -> isize {
        self.name.radius()
    }
}

#[derive(Debug, Clone)]
pub struct RelationElement {
    name: TextElement,
    attributes: Vec<AttributeElement>,
}

impl RelationElement {
    pub fn new(s: String, attributes: Vec<String>) -> RelationElement {
        RelationElement {
            name: TextElement::new(s),
            attributes: attributes
                .into_iter()
                .map(|a| AttributeElement::new(a))
                .collect(),
        }
    }
}

impl Draw for RelationElement {
    fn radius(&self) -> isize {
        self.name.radius()
            + self
                .attributes
                .iter()
                .map(|a| a.radius())
                .max()
                .unwrap_or(0)
    }
}
