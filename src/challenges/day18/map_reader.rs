use super::{Key, Tile};
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

impl Map {
    #[inline]
    // pub fn ptoi(&self, x: usize, y: usize) -> Option<usize> {
    //     let idx = y * self.width + x;
    //     if x < self.width && idx < self.data.len() {
    //         Some(idx)
    //     } else {
    //         None
    //     }
    // }
    // #[inline]
    // pub fn itop(&self, idx: usize) -> (usize, usize) {
    //     (
    //         idx % self.width,
    //         idx / self.width,
    //     )
    // }

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
