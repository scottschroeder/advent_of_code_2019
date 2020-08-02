use self::keys::{Key, KeySet};
use crate::util::{digits_to_int, parse_digits};
use anyhow::Result;
use std::fmt;
use std::fmt::{Error, Formatter};
use std::hint::unreachable_unchecked;
use std::iter;

pub fn part1(input: &str) -> Result<String> {
    let m = map_reader::Map::parse(input);
    //debug!(slog_scope::logger(), "{:#?}", m);
    let g = graph::CaveGraph::from_map(m);
    //debug!(slog_scope::logger(), "{}", g.dot());
    //return Ok(format!("{}", g.dot()));
    g.dijkstra(g.start());
    Ok(format!("{}", 0))
}

pub fn part2(input: &str) -> Result<String> {
    let e = KeySet::new();
    debug!(slog_scope::logger(), "{:?}", e);
    Ok(format!("{}", 0))
}

mod keys {
    use std::fmt;
    #[derive(Clone, Copy, PartialEq)]
    pub struct Key(u32);

    impl From<char> for Key {
        fn from(c: char) -> Self {
            let idx = (c.to_ascii_lowercase() as u8 - 'a' as u8) as usize;
            assert!(idx < 32);
            let a = u32::MAX >> idx;
            let b = u32::MAX >> (idx + 1);
            Key(a ^ b)
        }
    }

    impl From<Key> for char {
        fn from(k: Key) -> Self {
            ((k.0.leading_zeros() as u8) + ('a' as u8)) as char
        }
    }

    impl fmt::Debug for Key {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", char::from(*self))
        }
    }

    #[derive(Clone, Copy, PartialEq)]
    pub struct KeySet(u32);

    impl KeySet {
        pub fn new() -> Self {
            KeySet(0)
        }
        #[inline]
        pub fn insert(self, k: Key) -> KeySet {
            KeySet(self.0 | k.0)
        }
        #[inline]
        pub fn contains(self, k: Key) -> bool {
            (self.0 & k.0) > 0
        }
    }

    impl fmt::Debug for KeySet {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "KeySet(")?;
            for c in 'a'..='z' {
                if self.contains(c.into()) {
                    write!(f, "{}", c)?;
                }
            }
            write!(f, ")")?;
            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn check_key_reflective() {
            for c in 'a'..='z' {
                let k = Key::from(c);
                let rc = char::from(k);
                assert_eq!(rc, c);
            }
            for (cap_c, c) in ('A'..='Z').into_iter().zip('a'..='z') {
                let k = Key::from(cap_c);
                let rc = char::from(k);
                assert_eq!(rc, c);
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Tile {
    Wall,
    Space,
    Start,
    Door(Key),
    Key(Key),
}

mod graph {
    use super::map_reader::Map;
    use super::{Key, KeySet, Tile};

    use petgraph::stable_graph::{NodeIndex, StableGraph,};
    use petgraph::visit::{EdgeRef, IntoNodeReferences};
    use std::ops::Add;

    #[derive(Debug, Clone, Copy, PartialEq)]
    enum PathCost {
        Path(u32),
        Block,
    }

    impl Default for PathCost {
        fn default() -> Self {
            PathCost::Path(0)
        }
    }

    impl Add<PathCost> for PathCost {
        type Output = PathCost;
        fn add(self, rhs: PathCost) -> Self::Output {
            match (self, rhs) {
                (PathCost::Path(a), PathCost::Path(b)) => PathCost::Path(a + b),
                _ => PathCost::Block,
            }
        }
    }
    impl PartialOrd for PathCost {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(match (self, other) {
                // (None, None) => std::cmp::Ordering::Equal,
                // (None, Some(_)) => std::cmp::Ordering::Greater,
                // (Some(_), None) => std::cmp::Ordering::Less,
                // (Some(a), Some(b)) => a.cmp(&b),
                (PathCost::Block, PathCost::Block) => std::cmp::Ordering::Equal,
                (PathCost::Block, PathCost::Path(_)) => std::cmp::Ordering::Greater,
                (PathCost::Path(_), PathCost::Block) => std::cmp::Ordering::Less,
                (PathCost::Path(a), PathCost::Path(b)) => a.cmp(b),
            })
        }
    }

    #[derive(Debug, Clone)]
    pub(crate) struct ExploreState {
        pos: NodeIndex,
        length: u32,
        keys: KeySet,
    }

    #[derive(Default)]
    pub(crate) struct CaveGraph {
        inner: StableGraph<Tile, u8, petgraph::Undirected>,
        start: NodeIndex,
    }

    impl CaveGraph {
        pub(crate) fn start(&self) -> ExploreState {
            ExploreState {
                pos: self.start,
                length: 0,
                keys: KeySet::new(),
            }
        }
        pub(crate) fn from_map(m: Map) -> Self {
            let mut node_idx = Vec::with_capacity(m.data.len());
            let mut g = StableGraph::default();
            let mut start = None;
            for t in &m.data {
                let idx = g.add_node(*t);
                node_idx.push(idx);
                if *t == Tile::Start {
                    start = Some(idx);
                }
            }
            {
                let mut add_edge = |src: usize, dst: usize, t: Tile| {
                    if t != Tile::Wall {
                        g.add_edge(node_idx[src], node_idx[dst], 1);
                    }
                };
                for lrn in m.walk_lr() {
                    let (idx, node) = lrn.node;
                    if node == Tile::Wall {
                        continue;
                    }
                    if let Some((nidx, t)) = lrn.l {
                        add_edge(idx, nidx, t);
                    }
                    if let Some((nidx, t)) = lrn.r {
                        add_edge(idx, nidx, t);
                    }
                }
            }

            let mut cg = CaveGraph {
                inner: g,
                start: start.expect("graph must have a start tile"),
            };
            cg.trim_walls();
            cg
        }
        fn trim_walls(&mut self) {
            self.inner
                .retain_nodes(|g, n| *(g.node_weight(n).unwrap()) != Tile::Wall);
        }
        pub(crate) fn dot(&self) -> String {
            format!("{:?}", petgraph::dot::Dot::new(&self.inner))
        }

        pub(crate) fn dijkstra(&self, explore: ExploreState) {
            let log = slog_scope::logger();
            let all_keys = self.inner.node_references()
                .filter_map(|(_, t)| {
                    match t {
                        Tile::Key(k) => Some(*k),
                        _ => None,
                    }
                })
                .fold(KeySet::new(), |acc, k| acc.insert(k));
            debug!(log, "Looking for all keys in {:?}", all_keys);
            let mut shortest_distance = None;

            let mut q = vec![explore];

            while let Some(explore) = q.pop() {
                debug!(log, "TOTAL: {} Current {:?}", q.len(), explore);
                let seen = explore.keys;
                let mut stack = Vec::new();

                let m = petgraph::algo::dijkstra(&self.inner, explore.pos, None, |e| {
                    let src = e.source();
                    let src_t = self.inner.node_weight(src).unwrap();
                    let dst = e.target();
                    let dst_t = self.inner.node_weight(dst).unwrap();

                    // Walls, should never happen
                    if let Tile::Wall = src_t {
                        unreachable!("we can not start from inside a wall");
                    }
                    if let Tile::Wall = dst_t {
                        unreachable!("we can not end up inside a wall");
                    }

                    // Don't move past a new key
                    if let Tile::Key(k) = src_t {
                        if !seen.contains(*k) {
                            return PathCost::Block;
                        }
                    }

                    // Pick up new keys
                    if let Tile::Key(k) = dst_t {
                        if !seen.contains(*k) {
                            stack.push((dst, *k));
                        }
                    }

                    // Don't go to a door we don't have a key for
                    if let Tile::Door(k) = dst_t {
                        if !seen.contains(*k) {
                            return PathCost::Block;
                        }
                    }
                    PathCost::Path(*(e.weight()) as u32)
                });
                let next_keys = stack
                    .into_iter()
                    .filter_map(|(n, k)| {
                        let c = m.get(&n).cloned().unwrap_or(PathCost::Block);
                        match c {
                            PathCost::Path(c) => Some(ExploreState {
                                pos: n,
                                length: c + explore.length,
                                keys: seen.insert(k),
                            }),
                            PathCost::Block => None,
                        }
                    })
                    .filter(|e| {
                        if e.keys == all_keys {
                            let prev = shortest_distance.get_or_insert(e.length);
                            if e.length < *prev {
                                *prev = e.length;
                            }
                            false
                        } else {
                        true
                        }
                    })
                    .collect::<Vec<_>>();

                //debug!(log, "Dijkstra {:?}", m);
                q.extend(next_keys);
            }
            info!(log, "Shortest Path {:?}", shortest_distance);
        }

        pub(crate) fn dfs(&self, explore: ExploreState) {
            use petgraph::visit::{depth_first_search, Control, DfsEvent};
            let log = slog_scope::logger();
            let mut dist = explore.length;
            let seen = explore.keys;
            let mut stack = Vec::new();

            depth_first_search(&self.inner, Some(explore.pos), |event| {
                debug!(slog_scope::logger(), "{:?}", event);
                match event {
                    // DfsEvent::Discover(n, _) => {
                    //     debug!(log, "Discover {:?}", self.inner.node_weight(n).unwrap());
                    // }
                    DfsEvent::TreeEdge(src, dst) | DfsEvent::CrossForwardEdge(src, dst) => {
                        info!(
                            log,
                            "Edge {:?} -> {:?}",
                            self.inner.node_weight(src).unwrap(),
                            self.inner.node_weight(dst).unwrap()
                        );
                        dist += 1;

                        let t = self.inner.node_weight(dst).unwrap();
                        match t {
                            Tile::Wall => unreachable!("we can not end up inside a wall"),
                            Tile::Space | Tile::Start => Control::<()>::Continue,
                            Tile::Door(k) => {
                                if seen.contains(*k) {
                                    Control::Continue
                                } else {
                                    Control::Prune
                                }
                            }
                            Tile::Key(k) => {
                                if seen.contains(*k) {
                                    Control::Continue
                                } else {
                                    let e = ExploreState {
                                        pos: dst,
                                        length: dist,
                                        keys: seen.insert(*k),
                                    };
                                    stack.push(e);
                                    Control::Prune
                                }
                            }
                        }
                    }
                    DfsEvent::BackEdge(n, n1) => {
                        debug!(
                            log,
                            "BackEdge {:?} -> {:?}",
                            self.inner.node_weight(n).unwrap(),
                            self.inner.node_weight(n1).unwrap()
                        );
                        dist -= 1;
                        Control::Continue
                    }
                    // DfsEvent::CrossForwardEdge(n, n1) => {
                    //     error!(log, "CrossForwardEdge {:?} -> {:?}", self.inner.node_weight(n).unwrap(), self.inner.node_weight(n1).unwrap());
                    //     panic!("I dont know what to do with this")
                    // }
                    // DfsEvent::Finish(n, _) => {
                    //     debug!(log, "Finish {:?}", self.inner.node_weight(n).unwrap());
                    // }
                    _ => Control::Continue,
                }

                // if let DfsEvent::TreeEdge(src, dst) = event {
                // } else {
                //     Control::Continue
                // }
            });
            debug!(log, "Stack {:?}", stack);
        }
    }
}

mod map_reader {
    use super::{Key, Tile};

    #[derive(Debug)]
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
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::challenges::test::*;

    pub(crate) const EXAMPLES: [&str; 6] = [
        DAY18_EX1,
        DAY18_EX2,
        DAY18_EX3,
        DAY18_EX4,
        DAY18_EX5,
        DAY18_INPUT,
    ];

    #[test]
    fn day18part1() {
        assert_eq!(part1(DAY18_INPUT).unwrap().as_str(), "0")
    }

    #[test]
    fn day18part2() {
        assert_eq!(part2(DAY18_INPUT).unwrap().as_str(), "0")
    }
}
