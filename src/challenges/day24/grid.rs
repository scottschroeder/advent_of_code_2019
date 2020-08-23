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

impl RPoint {
    pub fn iter(self) -> AdjPoints {
        self.into_iter()
    }
}

impl IntoIterator for RPoint {
    type Item = RPoint;

    type IntoIter = AdjPoints;

    fn into_iter(self) -> Self::IntoIter {
        AdjPoints {
            p: self,
            compass: Compass::default(),
            idx: 0,
        }
    }
}

pub fn demo() {
    let cases = &[
        (
            "19",
            RPoint {
                p: Point::new(3, 3),
                r: 1,
            },
        ),
        (
            "G",
            RPoint {
                p: Point::new(1, 1),
                r: 0,
            },
        ),
        (
            "D",
            RPoint {
                p: Point::new(3, 0),
                r: 0,
            },
        ),
        (
            "E",
            RPoint {
                p: Point::new(4, 0),
                r: 0,
            },
        ),
        (
            "14",
            RPoint {
                p: Point::new(3, 2),
                r: 1,
            },
        ),
        (
            "N",
            RPoint {
                p: Point::new(3, 2),
                r: 0,
            },
        ),
        (
            "L",
            RPoint {
                p: Point::new(1, 2),
                r: 0,
            },
        ),
    ];
    for (n, rp) in cases {
        let adj = rp.iter().collect::<Vec<_>>();
        println!("{}: {:?}", n, adj);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Recurse {
    None,
    In,
    Out,
}

fn is_recurse(p: Point) -> Recurse {
    if p.x < 0 || p.x >= GRID_SIZE as i32 || p.y < 0 || p.y >= GRID_SIZE as i32 {
        Recurse::Out
    } else if p.x == 2 && p.y == 2 {
        Recurse::In
    } else {
        Recurse::None
    }
}

impl Direction {
    fn delta(self) -> Point {
        match self {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        }
        .into()
    }
}

struct Compass(Option<Direction>);
impl Default for Compass {
    fn default() -> Self {
        Compass(Some(Direction::Up))
    }
}

impl Iterator for Compass {
    type Item = Direction;

    fn next(&mut self) -> Option<Self::Item> {
        let r = self.0;
        self.0 = match self.0 {
            Some(Direction::Up) => Some(Direction::Right),
            Some(Direction::Down) => Some(Direction::Left),
            Some(Direction::Left) => None,
            Some(Direction::Right) => Some(Direction::Down),
            None => None,
        };
        r
    }
}

struct AdjPoints {
    p: RPoint,
    compass: Compass,
    idx: i32,
}

impl Iterator for AdjPoints {
    type Item = RPoint;

    fn next(&mut self) -> Option<Self::Item> {
        let d = self.compass.0?;
        let p_offset = self.p.p + d.delta();
        let recurse = is_recurse(p_offset);
        let p_true = match (d, recurse) {
            (_, Recurse::None) => p_offset,
            (Direction::Up, Recurse::Out) => Point::new(2, 1),
            (Direction::Down, Recurse::Out) => Point::new(2, 3),
            (Direction::Left, Recurse::Out) => Point::new(1, 2),
            (Direction::Right, Recurse::Out) => Point::new(3, 2),
            (Direction::Up, Recurse::In) => Point::new(self.idx, GRID_SIZE as i32 - 1),
            (Direction::Down, Recurse::In) => Point::new(self.idx, 0),
            (Direction::Left, Recurse::In) => Point::new(GRID_SIZE as i32 - 1, self.idx),
            (Direction::Right, Recurse::In) => Point::new(0, self.idx),
        };

        self.idx += 1;
        let max_idx = if let Recurse::In = recurse {
            GRID_SIZE as i32 - 1
        } else {
            0
        };

        if self.idx > max_idx {
            self.idx = 0;
            self.compass.next();
        }

        let depth_change = match recurse {
            Recurse::None => 0,
            Recurse::In => -1,
            Recurse::Out => 1,
        };
        Some(RPoint {
            p: p_true,
            r: self.p.r + depth_change,
        })
    }
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

pub struct RecursiveGrid {
    up: Vec<Grid>,
    down: Vec<Grid>,
}

impl Default for RecursiveGrid {
    fn default() -> Self {
        RecursiveGrid {
            up: vec![],
            down: vec![],
        }
    }
}

/*
 -2 1
 -1 0
 ----
  0 0
  1 1
  2 2


*/

enum RecurseDirection {
    Up(usize),
    Down(usize),
}

impl From<i64> for RecurseDirection {
    fn from(depth: i64) -> Self {
        if depth < 0 {
            RecurseDirection::Down((depth * -1 - 1) as usize)
        } else {
            RecurseDirection::Up(depth as usize)
        }
    }
}

fn extend_with<T, F>(v: &mut Vec<T>, idx: usize, f: F)
where
    T: Clone,
    F: FnOnce() -> T,
{
    let need = idx as i64 + 1 - v.len() as i64;
    if need > 0 {
        v.extend(std::iter::repeat(f()).take(need as usize))
    }
}

impl RecursiveGrid {
    pub fn from_map(s: &str) -> Result<RecursiveGrid> {
        let mut rg = RecursiveGrid::default();
        rg.up.push(Grid::from_map(s)?);
        Ok(rg)
    }
    fn trim(&mut self) {
        while self.up.last().map(|g| g.count()).unwrap_or(1) == 0 {
            self.up.pop();
        }
        while self.down.last().map(|g| g.count()).unwrap_or(1) == 0 {
            self.down.pop();
        }
    }
    pub fn count(&self) -> usize {
        let mut n = 0;
        for g in &self.up {
            n += g.count()
        }
        for g in &self.down {
            n += g.count()
        }
        n
    }
    fn get_grid(&self, depth: i64) -> Option<&Grid> {
        let rd = RecurseDirection::from(depth);
        match rd {
            RecurseDirection::Up(idx) => self.up.get(idx),
            RecurseDirection::Down(idx) => self.down.get(idx),
        }
    }
    fn get_mut_grid(&mut self, depth: i64) -> Option<&mut Grid> {
        let rd = RecurseDirection::from(depth);
        match rd {
            RecurseDirection::Up(idx) => {
                extend_with(&mut self.up, idx, || Grid::default());
                self.up.get_mut(idx)
            }
            RecurseDirection::Down(idx) => {
                extend_with(&mut self.down, idx, || Grid::default());
                self.down.get_mut(idx)
            }
        }
    }
    fn get(&self, rp: RPoint) -> Tile {
        let RPoint { p, r } = rp;
        self.get_grid(r).map(|g| g.get(p)).unwrap_or(Tile::Space)
    }
    fn adj_bugs(&self, rp: RPoint) -> u8 {
        let mut n = 0;
        for adj in rp.iter() {
            match self.get(adj) {
                Tile::Space => {}
                Tile::Bug => n += 1,
            }
        }
        n
    }
    fn make_bug(&mut self, rp: RPoint) {
        let RPoint { p, r } = rp;
        if let Some(g) = self.get_mut_grid(r) {
            g.make_bug(p);
        }
    }
    pub fn update(&mut self) {
        let mut new = RecursiveGrid::default();
        let low = self.down.len() as i64 * -1 - 1;
        let high = self.up.len() as i64 + 1;
        for rp in (low..high)
            .flat_map(|r| points(true).map(move |p| RPoint { p, r }))
            .filter_map(|rp| {
                if let Tile::Bug = new_tile(self.adj_bugs(rp), self.get(rp)) {
                    Some(rp)
                } else {
                    None
                }
            })
        {
            new.make_bug(rp);
        }
        new.trim();
        std::mem::swap(self, &mut new);
    }
}
impl fmt::Display for RecursiveGrid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (r, g) in self
            .down
            .iter()
            .enumerate()
            .rev()
            .map(|(idx, g)| (-1 * (idx as i64 + 1), g))
            .chain(self.up.iter().enumerate().map(|(idx, g)| (idx as i64, g)))
        {
            write!(f, "Depth {}:\n{}\n\n", r, g)?;
        }
        Ok(())
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

fn points(recursive: bool) -> impl Iterator<Item = Point> {
    let skip = if recursive {
        Some(Point::new(2, 2))
    } else {
        None
    };
    (0..GRID_SQUARE)
        .map(|idx| {
            let x = idx % GRID_SIZE;
            let y = idx / GRID_SIZE;
            Point::from((x as i32, y as i32))
        })
        .filter(move |p| skip.as_ref().map(|s| p != s).unwrap_or(true))
}

#[inline]
fn new_tile(adj: u8, current: Tile) -> Tile {
    match (adj, current) {
        (1, Tile::Bug) => Tile::Bug,
        (1, Tile::Space) => Tile::Bug,
        (2, Tile::Space) => Tile::Bug,
        _ => Tile::Space,
    }
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
    pub fn count(&self) -> usize {
        self.map.iter().filter(|t| **t == Tile::Bug).count()
    }
    fn get(&self, p: Point) -> Tile {
        if let Some(idx) = ptoi(p) {
            self.map[idx]
        } else {
            Tile::Space
        }
    }
    fn make_bug(&mut self, p: Point) {
        if let Some(idx) = ptoi(p) {
            self.map[idx] = Tile::Bug;
        }
    }
    pub fn adj_bugs(&self, p: Point) -> u8 {
        let mut n = 0;
        for adj in Compass::default().map(|d| d.delta()) {
            match self.get(p + adj) {
                Tile::Space => {}
                Tile::Bug => n += 1,
            }
        }
        n
    }
    pub fn update(&mut self) {
        let mut new = Grid::default();
        for idx in points(false)
            .map(|p| {
                let adj = self.adj_bugs(p);
                new_tile(adj, self.get(p))
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
        let mut idx = 0;
        let mut iter_p = Point::new(0, 0);
        for c in s.chars() {
            match c {
                '\n' => {
                    if iter_p.x as usize != GRID_SIZE {
                        return Err(ah!("grid was not {} wide: {:?}", GRID_SIZE, iter_p));
                    }
                    iter_p.x = 0;
                    iter_p.y += 1;
                    continue;
                }
                '#' => {
                    map.map[idx] = Tile::Bug;
                }
                '.' => {}
                _ => return Err(ah!("unknown tile: {:?}", c)),
            }
            iter_p.x += 1;
            idx += 1;
        }
        Ok(map)
    }
}
