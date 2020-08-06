use anyhow::Result;

pub fn part1(input: &str) -> Result<String> {
    let m = map::Map::parse(input);
    let portals = m.labels();
    log::debug!("{:#?}", portals);
    Ok(format!("{}", 0))
}

pub fn part2(input: &str) -> Result<String> {
    Ok(format!("{}", 0))
}

mod graph {
    use petgraph::stable_graph::{NodeIndex, StableGraph};

    pub(crate) struct DonutGraph {
        inner: StableGraph<(), u32, petgraph::Undirected>,
    }
}

mod map {
    use std::collections::HashMap;
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub(crate) enum Tile {
        Dead,
        Label(char),
        Wall,
        Space,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub(crate) struct Portal((char, char));

    #[derive(Clone, Copy)]
    enum Direction {
        Up,
        Down,
        Left,
        Right,
    }

    impl Direction {
        fn rotate(self) -> Direction {
            match self {
                Direction::Up => Direction::Right,
                Direction::Down => Direction::Left,
                Direction::Left => Direction::Up,
                Direction::Right => Direction::Down,
            }
        }
        fn invert(self) -> Direction {
            match self {
                Direction::Up => Direction::Down,
                Direction::Down => Direction::Up,
                Direction::Left => Direction::Right,
                Direction::Right => Direction::Left,
            }
        }
        fn translate(self, x: i32, y: i32) -> (i32, i32) {
            match self {
                Direction::Up => (x, y - 1),
                Direction::Down => (x, y + 1),
                Direction::Left => (x - 1, y),
                Direction::Right => (x + 1, y),
            }
        }
    }

    pub(crate) struct Map {
        data: Vec<Tile>,
        width: usize,
    }

    impl Map {
        pub(crate) fn parse(s: &str) -> Map {
            let mut width = None;
            let mut data = Vec::with_capacity(s.len());

            for (idx, c) in s.chars().enumerate() {
                let t = match c {
                    '\n' => {
                        width.get_or_insert(idx);
                        continue;
                    }
                    '#' => Tile::Wall,
                    '.' => Tile::Space,
                    'A'..='Z' => Tile::Label(c),
                    ' ' => Tile::Dead,
                    _ => unreachable!("char {:?} does not belong in input", c),
                };
                data.push(t);
            }
            Map {
                width: width.unwrap_or_else(|| data.len()),
                data,
            }
        }
        pub(crate) fn labels(&self) -> HashMap<Portal, Vec<usize>> {
            let mut m = HashMap::new();
            for (p, spc) in self
                .data
                .iter()
                .enumerate()
                .filter_map(|(idx, t)| self.try_label(idx))
            {
                let v = m.entry(p).or_insert_with(|| Vec::with_capacity(2));
                v.push(spc);
            }
            m
        }

        fn try_label(&self, idx: usize) -> Option<(Portal, usize)> {
            let l1 = if let Tile::Label(c) = self.data[idx] {
                c
            } else {
                return None;
            };

            let (x, y) = self.itop(idx);

            let fetch = |d: Direction| self.ptoi(d.translate(x, y)).map(|i| self.data[i]);

            let mut l2 = None;
            let mut adj = None;
            let mut d = Direction::Up;
            for _ in 0..3 {
                let t = fetch(d)?;
                match t {
                    Tile::Dead => d = d.rotate(),
                    Tile::Label(c) => {
                        l2 = Some(c);
                        d = d.invert();
                    }
                    Tile::Wall => panic!("a wall should never be adjacent to a label"),
                    Tile::Space => {
                        adj = Some(d);
                        d = d.invert();
                    }
                }
            }
            let l2 = l2?;
            let adj = adj?;
            let spc = self.ptoi(d.translate(x, y))?;

            Some(match adj {
                Direction::Up => (Portal((l1, l2)), spc),
                Direction::Down => (Portal((l2, l1)), spc),
                Direction::Left => (Portal((l1, l2)), spc),
                Direction::Right => (Portal((l2, l1)), spc),
            })
        }

        #[inline]
        fn ptoi(&self, p: (i32, i32)) -> Option<usize> {
            let (x, y) = p;
            if y < 0 || x < 0 {
                return None;
            }
            let x = x as usize;
            let y = y as usize;
            let idx = y * self.width + x;
            if x < self.width && idx < self.data.len() {
                Some(idx)
            } else {
                None
            }
        }
        #[inline]
        fn itop(&self, idx: usize) -> (i32, i32) {
            ((idx % self.width) as i32, (idx / self.width) as i32)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::challenges::test::*;

    #[test]
    fn verify_part1() {
        assert_eq!(part1(DAY20_INPUT).unwrap().as_str(), "0")
    }

    #[test]
    fn verify_part2() {
        assert_eq!(part2(DAY20_INPUT).unwrap().as_str(), "0")
    }

    #[test]
    fn verify_p1_ex1() {
        assert_eq!(part1(DAY20_EX1).unwrap().as_str(), "23")
    }
    #[test]
    fn verify_p1_ex2() {
        assert_eq!(part1(DAY20_EX2).unwrap().as_str(), "58")
    }
}
