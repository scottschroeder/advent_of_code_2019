use crate::display::Point;
use anyhow::{anyhow as ah, Result};
use std::fmt;
const GRID_SIZE: usize = 5;
const GRID_SQUARE: usize = GRID_SIZE * GRID_SIZE;

type GridArray = [Tile; GRID_SQUARE];


#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct RPoint {
    p: Point,
    r: i64,
}

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

struct AdjPoints {
    p: RPoint,
    direction: Direction,
    idx: usize,
}


impl fmt::Debug for RPoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({},{},{})", self.p.x, self.p.y, self.r)
    }
}



#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tile {
    Space,
    Bug,
}

impl Default for Tile {
    fn default() -> Self {
        Tile::Space
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Grid {
    map: GridArray,
}

impl Default for Grid {
    fn default() -> Self {
        Grid {
            map: [Tile::Space; GRID_SQUARE],
        }
    }
}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in 0..GRID_SIZE {
            if y != 0 {
                write!(f, "\n")?;
            }
            for x in 0..GRID_SIZE {
                match self.map[y * GRID_SIZE + x] {
                    Tile::Space => write!(f, ".")?,
                    Tile::Bug => write!(f, "#")?,
                }
            }
        }
        Ok(())
    }
}

fn points() -> impl Iterator<Item = Point> {
    (0..GRID_SQUARE).map(|idx| {
        let x = idx % GRID_SIZE;
        let y = idx / GRID_SIZE;
        Point::from((x as i32, y as i32))
    })
}

#[inline]
fn ptoi(p: Point) -> Option<usize> {
    let (x, y) = if p.x >= 0 && p.y >= 0 {
        (p.x as usize, p.y as usize)
    } else {
        return None;
    };
    if x < GRID_SIZE && y < GRID_SIZE {
        Some(y * GRID_SIZE + x)
    } else {
        None
    }
}

impl Grid {
    pub fn biodiversity(&self) -> u32 {
        let mut bd = 0;
        for (idx, t) in self.map.iter().enumerate() {
            if let Tile::Bug = t {
                bd += 1 << idx;
            }
        }
        bd
    }
    #[inline]
    fn kill_bug(&mut self, p: Point) {
        if let Some(idx) = ptoi(p) {
            self.map[idx] = Tile::Space;
        }
    }
    #[inline]
    fn make_bug(&mut self, p: Point) {
        if let Some(idx) = ptoi(p) {
            self.map[idx] = Tile::Bug;
        }
    }
    fn get(&self, p: Point) -> Tile {
        if let Some(idx) = ptoi(p) {
            self.map[idx]
        } else {
            Tile::Space
        }
    }
    pub fn adj_bugs(&self, p: Point) -> u8 {
        let mut n = 0;
        for adj in &[(0, 1), (0, -1), (-1, 0), (1, 0)] {
            let neighbor = p + Point::from(adj);
            match self.get(neighbor) {
                Tile::Space => {}
                Tile::Bug => n += 1,
            }
        }
        n
    }
    pub fn update(&mut self) {
        let mut new = Grid::default();
        for idx in points()
            .map(|p| {
                let adj = self.adj_bugs(p);
                match (adj, self.get(p)) {
                    (1, Tile::Bug) => Tile::Bug,
                    (1, Tile::Space) => Tile::Bug,
                    (2, Tile::Space) => Tile::Bug,
                    _ => Tile::Space,
                }
            })
            .enumerate()
            .filter_map(|(idx, t)| if let Tile::Bug = t { Some(idx) } else { None })
        {
            new.map[idx] = Tile::Bug;
        }
        std::mem::swap(&mut self.map, &mut new.map);
    }
    pub fn from_map(s: &str) -> Result<Grid> {
        let mut map = Grid::default();
        let mut p = Point::new(0, 0);
        for c in s.chars() {
            match c {
                '\n' => {
                    if p.x as usize != GRID_SIZE {
                        return Err(ah!("grid was not {} wide: {:?}", GRID_SIZE, p));
                    }
                    p.x = 0;
                    p.y += 1;
                    continue;
                }
                '#' => map.make_bug(p),
                '.' => {}
                _ => return Err(ah!("unknown tile: {:?}", c)),
            }
            p.x += 1;
        }
        Ok(map)
    }
}
