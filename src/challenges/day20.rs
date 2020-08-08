use anyhow::Result;

pub fn part1(input: &str) -> Result<String> {
    let m = map::Map::parse(input);
    let g = graph::DonutGraph::from_map(m)?;
    let min = g.shortest_path().unwrap();
    //Ok(format!("{}", g.dot()))
    Ok(format!("{}", min))
}

pub fn part2(input: &str) -> Result<String> {
    let m = map::Map::parse(input);

    Ok(format!("{}", 0))
}

mod graph {
    use super::map::{Map, Portal, Tile};
    use petgraph::graph::{Graph, NodeIndex};
    use std::collections::BTreeMap;

    pub(crate) struct DonutGraph {
        inner: Graph<(), u32, petgraph::Undirected>,
        aa: NodeIndex,
        zz: NodeIndex,
    }

    impl DonutGraph {
        pub(crate) fn shortest_path(&self) -> Option<usize> {
            let m = petgraph::algo::dijkstra(&self.inner, self.aa, Some(self.zz), |_| 1usize);
            m.get(&self.zz).cloned()
        }
        pub(crate) fn from_map(m: Map) -> anyhow::Result<Self> {
            let mut g = Graph::default();
            let mut node_map = BTreeMap::new();
            let mut aa = None;
            let mut zz = None;
            for (idx, t) in m.data.iter().enumerate() {
                if let Tile::Space = *t {
                    let nidx = g.add_node(());
                    node_map.insert(idx, nidx);
                }
            }
            let ntoe = |src, dst| {
                node_map
                    .get(&src)
                    .and_then(|s| node_map.get(&dst).map(|d| (s, d)))
                    .unwrap()
            };
            for (src, dst) in m.edges().map(|(src, dst)| ntoe(src, dst)) {
                g.add_edge(*src, *dst, 1);
            }

            log::trace!("{:#?}", node_map);

            for (p, gates) in m.labels()? {
                log::trace!("{:?}, v: {:?}", p, gates);
                if p == Portal(('A', 'A')) {
                    aa = Some(node_map.get(&gates.outer.unwrap()).unwrap());
                    log::trace!("set aa to {:?}", aa);
                } else if p == Portal(('Z', 'Z')) {
                    zz = Some(node_map.get(&gates.outer.unwrap()).unwrap());
                    log::trace!("set zz to {:?}", zz);
                } else {
                    let inner = gates.inner.unwrap();
                    let outer = gates.outer.unwrap();
                    let (src, dst) = ntoe(inner, outer);
                    log::trace!(
                        "insert portal edge {:?} {:?} {:?} -> {:?}",
                        p,
                        gates,
                        src,
                        dst
                    );
                    g.add_edge(*src, *dst, 1);
                }
            }

            Ok(DonutGraph {
                inner: g,
                aa: *aa.expect("no AA in map"),
                zz: *zz.expect("no ZZ in map"),
            })
        }

        pub(crate) fn dot(&self) -> String {
            format!("{:?}", petgraph::dot::Dot::new(&self.inner))
        }
    }
}

mod map {
    use std::collections::HashMap;
    use std::fmt;
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub(crate) enum Tile {
        Dead,
        Label(char),
        Wall,
        Space,
    }

    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub(crate) struct Portal(pub (char, char));

    impl fmt::Debug for Portal {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "Portal({}{})", self.0.0, self.0.1)
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Default)]
    pub(crate) struct PortalGates {
        pub outer: Option<usize>,
        pub inner: Option<usize>,
    }

    #[derive(Debug, Clone, Copy)]
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

    #[derive(Debug)]
    pub(crate) struct Map {
        pub(crate) data: Vec<Tile>,
        pub(crate) width: usize,
        pub(crate) bottom_wall: usize,
    }

    struct Pair<T> {
        first: Option<T>,
        second: Option<T>,
    }

    impl<T> Pair<T> {
        fn new(first: T, second: T) -> Pair<T> {
            Pair {
                first: Some(first),
                second: Some(second),
            }
        }
    }

    impl<T> Iterator for Pair<T> {
        type Item = T;
        fn next(&mut self) -> Option<Self::Item> {
            self.first.take().or_else(|| self.second.take())
        }
    }

    impl Map {
        pub(crate) fn parse(s: &str) -> Map {
            let mut width = None;
            let mut data = Vec::with_capacity(s.len());
            let mut last_wall = 0;

            for (idx, c) in s.chars().enumerate() {
                let t = match c {
                    '\n' => {
                        width.get_or_insert(idx);
                        continue;
                    }
                    '#' => {
                        last_wall = data.len();
                        Tile::Wall
                    }
                    '.' => Tile::Space,
                    'A'..='Z' => Tile::Label(c),
                    ' ' => Tile::Dead,
                    _ => unreachable!("char {:?} does not belong in input", c),
                };
                data.push(t);
            }
            let width = width.unwrap_or_else(|| data.len());
            Map {
                data,
                width,
                bottom_wall: last_wall / width,
            }
        }
        pub(crate) fn labels(&self) -> anyhow::Result<HashMap<Portal, PortalGates>> {
            let mut m = HashMap::new();
            for (p, spc, is_outer) in self
                .data
                .iter()
                .enumerate()
                .filter_map(|(idx, _)| self.try_label(idx))
            {
                let v = m.entry(p).or_insert_with(|| PortalGates::default());
                let (x, y) = self.itop(spc);
                log::trace!(
                    "{:?} spc:{:?} p:({}, {}) outer:{:?}",
                    p,
                    spc,
                    x,
                    y,
                    is_outer
                );
                if is_outer {
                    v.outer = Some(spc)
                } else {
                    v.inner = Some(spc)
                }
            }
            for (p, g) in &m {
                if *p == Portal(('A', 'A')) || *p == Portal(('Z', 'Z')) {
                    continue;
                }
                if g.inner.is_none() || g.outer.is_none() {
                    return Err(anyhow::anyhow!(
                        "portal {:?} did not have two gates: {:?}",
                        p,
                        g
                    ));
                }
            }
            Ok(m)
        }

        pub(crate) fn edges(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
            let w = self.width;
            self.data
                .iter()
                .enumerate()
                .flat_map(move |(idx, t)| {
                    let r_idx = idx + 1;
                    let l_idx = idx + w;
                    let l = if l_idx < self.data.len() {
                        Some((idx, *t, l_idx, self.data[l_idx]))
                    } else {
                        None
                    };
                    let r = if r_idx % w != 0 && r_idx < self.data.len() {
                        Some((idx, *t, r_idx, self.data[r_idx]))
                    } else {
                        None
                    };

                    Pair::new(l, r)
                })
                .filter_map(|x| x)
                .filter_map(|(idx, it, jdx, jt)| {
                    if it == Tile::Space && jt == Tile::Space {
                        Some((idx, jdx))
                    } else {
                        None
                    }
                })
        }

        #[inline]
        fn is_outer_wall(&self, p: (i32, i32)) -> bool {
            /*
            ......
            ......
            ..##..
            ..##..
            ......
            ......
            */
            let (x, y) = p;
            x == 2 || y == 2 || y == self.bottom_wall as i32 || (x == (self.width as i32 - 3))
        }

        fn try_label(&self, idx: usize) -> Option<(Portal, usize, bool)> {
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
                let xy = d.translate(x, y);
                log::trace!(
                    "({}, {}) * {:?} => ({}, {}), {:?} => {:?}",
                    x,
                    y,
                    d,
                    xy.0,
                    xy.1,
                    self.data[idx],
                    t
                );
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
            let spc_p = adj.translate(x, y);
            let spc = self.ptoi(spc_p)?;
            let portal = match adj {
                Direction::Up => Portal((l1, l2)),
                Direction::Down => Portal((l2, l1)),
                Direction::Left => Portal((l1, l2)),
                Direction::Right => Portal((l2, l1)),
            };
            Some((portal, spc, self.is_outer_wall(spc_p)))
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
        assert_eq!(part1(DAY20_INPUT).unwrap().as_str(), "642")
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
