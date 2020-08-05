use super::{Key, Tile};
use anyhow::anyhow as ah;
use std::fmt;

#[derive(Debug, Clone)]
pub(crate) struct Map {
    pub(crate) data: Vec<Tile>,
    pub(crate) width: usize,
}

pub(crate) struct LRNeighbors {
    pub(crate) node: (usize, Tile),
    pub(crate) l: Option<(usize, Tile)>,
    pub(crate) r: Option<(usize, Tile)>,
}

const MAP_PATCH: [Tile; 9] = [
    Tile::Start,
    Tile::Wall,
    Tile::Start,
    Tile::Wall,
    Tile::Wall,
    Tile::Wall,
    Tile::Start,
    Tile::Wall,
    Tile::Start,
];

impl Map {
    #[inline]
    pub fn ptoi(&self, x: usize, y: usize) -> Option<usize> {
        let idx = y * self.width + x;
        if x < self.width && idx < self.data.len() {
            Some(idx)
        } else {
            None
        }
    }
    #[inline]
    pub fn itop(&self, idx: usize) -> (usize, usize) {
        (idx % self.width, idx / self.width)
    }

    pub fn split_map(&mut self) -> Result<(), anyhow::Error> {
        let starts = self
            .data
            .iter()
            .enumerate()
            .filter(|(_, t)| **t == Tile::Start)
            .map(|(x, _)| x)
            .collect::<Vec<_>>();

        for idx in starts {
            let (x, y) = self.itop(idx);
            if x == 0 || y == 0 {
                return Err(ah!("start position was at an edge"));
            }
            let tiles = (-1..=1)
                .flat_map(|dy| (-1..=1).map(move |dx| (x as i32 + dx, y as i32 + dy)))
                .map(|(x, y)| {
                    self.ptoi(x as usize, y as usize)
                        .ok_or_else(|| ah!("start neighbor was out of range"))
                        .and_then(|tidx| {
                            if tidx == idx || self.data[tidx] == Tile::Space {
                                Ok(tidx)
                            } else {
                                Err(ah!("start-adjacent tile was not a space"))
                            }
                        })
                })
                .collect::<Result<Vec<usize>, _>>()?;
            for (pidx, t) in tiles.iter().zip(MAP_PATCH.iter()) {
                self.data[*pidx] = *t;
            }
        }
        Ok(())
    }

    pub fn walk_lr(&self) -> impl Iterator<Item = LRNeighbors> + '_ {
        let w = self.width;
        self.data.iter().enumerate().map(move |(idx, t)| {
            let r_idx = idx + 1;
            let l_idx = idx + w;
            let l = if l_idx < self.data.len() {
                Some((l_idx, self.data[l_idx]))
            } else {
                None
            };
            let r = if r_idx % w != 0 {
                Some((r_idx, self.data[r_idx]))
            } else {
                None
            };
            LRNeighbors {
                node: (idx, *t),
                l,
                r,
            }
        })
    }
    pub fn parse(s: &str) -> Map {
        let mut width = None;
        let mut data = Vec::with_capacity(s.len());
        for (idx, c) in s.chars().enumerate() {
            let t = match c {
                '\n' => {
                    width.get_or_insert(idx);
                    continue;
                }
                '#' => Tile::Wall,
                '@' => Tile::Start,
                '.' => Tile::Space,
                'a'..='z' => Tile::Key(Key::from(c)),
                'A'..='Z' => Tile::Door(Key::from(c)),
                _ => unreachable!("char {:?} does not belong in input", c),
            };
            data.push(t);
        }
        Map {
            width: width.unwrap_or_else(|| data.len()),
            data,
        }
    }
}

impl fmt::Display for Map {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (idx, t) in self.data.iter().enumerate() {
            if idx > 0 && idx % self.width == 0 {
                write!(f, "\n")?;
            }
            match t {
                Tile::Wall => write!(f, "#")?,
                Tile::Space => write!(f, ".")?,
                Tile::Start => write!(f, "@")?,
                Tile::Door(k) => write!(f, "{}", char::from(*k).to_ascii_uppercase())?,
                Tile::Key(k) => write!(f, "{}", char::from(*k))?,
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::challenges::day18::test::EXAMPLES;

    #[test]
    fn parse_maps() {
        for e in EXAMPLES.into_iter() {
            Map::parse(e);
        }
    }
}
