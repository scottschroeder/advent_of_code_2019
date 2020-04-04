/*!
Display for terminal

Coordinates are integers (that may be negative)
The size can be unknown
*/

use std::fmt;
use std::iter::FromIterator;
use std::fmt::{Formatter, Error};

#[derive(Default)]
pub struct VON;

#[derive(Default)]
pub struct VOF;

pub trait VerticalOrientation: Default {
    fn offset(ymin: i32, ymax: i32, y: i32) -> i32;
    fn absolute(ymin: i32, ymax: i32, dy: i32) -> i32;
}

impl VerticalOrientation for VON {
    #[inline]
    fn offset(ymin: i32, ymax: i32, y: i32) -> i32 {
        ymax - y
    }
    #[inline]
    fn absolute(ymin: i32, ymax: i32, dy: i32) -> i32 {
        ymax - dy
    }
}

impl VerticalOrientation for VOF {
    #[inline]
    fn offset(ymin: i32, ymax: i32, y: i32) -> i32 {
        y - ymin
    }
    #[inline]
    fn absolute(ymin: i32, ymax: i32, dy: i32) -> i32 {
        dy + ymin
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl From<(i32, i32)> for Point {
    fn from(p: (i32, i32)) -> Self {
        Point { x: p.0, y: p.1 }
    }
}

impl From<&(i32, i32)> for Point {
    fn from(p: &(i32, i32)) -> Self {
        Point { x: p.0, y: p.1 }
    }
}

impl From<&Point> for Point {
    fn from(p: &Point) -> Self {
        *p
    }
}

impl Point {
    pub fn new(x: i32, y: i32) -> Point {
        Point { x, y }
    }
    pub fn step_dist(&self, other: &Point) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }
}

pub type ImageNormal<T> = Image<T, VON>;
pub type ImageFlip<T> = Image<T, VOF>;

pub struct Image<T, V> {
    frame: Frame,
    grid: bool,
    pub data: Vec<Option<T>>,
    v: V,
}

impl<T, V> Image<T, V> {
    pub fn width(&self) -> usize {
        self.frame.width() as usize
    }
}

impl<T: Clone, V: VerticalOrientation> Image<T, V> {
    pub fn display_grid(&mut self, enable: bool) {
        self.grid = enable;
    }
    pub fn create<'a, I, P>(iter: &'a I) -> Image<T, V>
        where
            &'a I: IntoIterator<Item=(&'a P, &'a T)>,
            &'a P: Into<Point>,
            T: 'a,
            P: 'a,
    {
        let mut frame = size_frame(iter);
        let mut img = Image {
            frame,
            grid: false,
            data: Vec::new(),
            v: V::default(),
        };
        img.update(iter);
        img
    }


    pub fn update<'a, I, P>(&mut self, iter: &'a I)
        where
            &'a I: IntoIterator<Item=(&'a P, &'a T)>,
            &'a P: Into<Point>,
            T: 'a,
            P: 'a,
    {
        self.data = vec![None; self.frame.len()];

        for (p, t) in iter {
            let pt = p.into();
            self.data[self.frame.index::<V>(pt)] = Some(t.clone());
        }
    }
}

impl<T: fmt::Display, V: VerticalOrientation> fmt::Display for Image<T, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let w = self.width() as usize;

        for (idx, v) in self.data.iter().enumerate() {
            if idx % w == 0 {
                let p = self.frame.point::<V>(idx);
                if self.grid {
                    write!(f, "\n{}\t", p.y)?;
                } else {
                    writeln!(f, "")?;
                }
            }
            if let Some(t) = v {
                write!(f, "{}", t)?;
            } else {
                write!(f, " ")?;
            }
        }
        if self.grid {
            write!(f, "\n\t")?;
            for x in self.frame.min_x..(self.frame.max_x + 1) {
                write!(f, "{}", (x % 10).abs())?;
            }
            writeln!(f, "")?;
        }
        Ok(())
    }
}

struct Frame {
    min_x: i32,
    max_x: i32,
    min_y: i32,
    max_y: i32,
}

impl Frame {
    #[inline]
    fn width(&self) -> i32 {
        self.max_x + 1 - self.min_x
    }
    #[inline]
    fn height(&self) -> i32 {
        self.max_y + 1 - self.min_y
    }
    #[inline]
    fn len(&self) -> usize {
        (self.width() * self.height()) as usize
    }
    #[inline]
    fn index<V: VerticalOrientation>(&self, p: Point) -> usize {
        let w = self.width();
        let dx = p.x - self.min_x;
        let dy = V::offset(self.min_y, self.max_y, p.y);
        let idx = (dx + w * dy) as usize;
        idx
    }
    #[inline]
    fn point<V: VerticalOrientation>(&self, idx: usize) -> Point {
        let idx = idx as i32;
        let w = self.width();
        let x = idx % w;
        let y = V::absolute(self.min_y, self.max_y, idx / w);
        Point {x, y}
    }
}

fn size_frame<'a, I, T: 'a, P>(iter: &'a I) -> Frame
    where
        &'a I: IntoIterator<Item=(&'a P, &'a T)>,
        P: 'a,
        &'a P: Into<Point>,
{
    let mut min_x = None;
    let mut max_x = None;
    let mut min_y = None;
    let mut max_y = None;
    for p in iter {
        let (p, _) = p;
        let Point { x, y } = p.into();

        min_x = Some(if let Some(mx) = min_x {
            std::cmp::min(mx, x)
        } else {
            x
        });
        max_x = Some(if let Some(mx) = max_x {
            std::cmp::max(mx, x)
        } else {
            x
        });
        min_y = Some(if let Some(my) = min_y {
            std::cmp::min(my, y)
        } else {
            y
        });
        max_y = Some(if let Some(my) = max_y {
            std::cmp::max(my, y)
        } else {
            y
        });
    }
    Frame {
        min_x: min_x.unwrap(),
        max_x: max_x.unwrap(),
        min_y: min_y.unwrap(),
        max_y: max_y.unwrap(),
    }
}
