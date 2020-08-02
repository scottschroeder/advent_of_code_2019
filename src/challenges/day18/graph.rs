use super::map_reader::Map;
use super::{Key, KeySet, Tile};
use std::collections::HashMap;

use petgraph::stable_graph::{NodeIndex, StableGraph};
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

trait ExploreQueue : Default {
    fn insert(&mut self, e: ExploreState);
    fn pop(&mut self) -> Option<ExploreState>;
    fn len(&self) -> usize;
}

#[derive(Default)]
struct ExploreStack(Vec<ExploreState>);

impl ExploreQueue for ExploreStack {
    fn insert(&mut self, e: ExploreState) {
        self.0.push(e)
    }
    fn pop(&mut self) -> Option<ExploreState> {
        self.0.pop()
    }
    fn len(&self) -> usize {
        self.0.len()
    }
}

#[derive(Default)]
struct ExploreMinHeap(std::collections::BinaryHeap<std::cmp::Reverse<ExploreState>>);

impl ExploreQueue for ExploreMinHeap {
    fn insert(&mut self, e: ExploreState) {
        self.0.push(std::cmp::Reverse(e))
    }
    fn pop(&mut self) -> Option<ExploreState> {
        self.0.pop().map(|re| re.0)
    }
    fn len(&self) -> usize {
        self.0.len()
    }
    
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ExploreState {
    pos: NodeIndex,
    length: u32,
    keys: KeySet,
}

impl ExploreState {
    fn cache_key(self) -> (SingleCacheKey, u32) {
        (SingleCacheKey((self.pos, self.keys)), self.length)
    }
}

impl PartialOrd for ExploreState {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for ExploreState {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.keys.len().cmp(&other.keys.len()) {
            std::cmp::Ordering::Less => std::cmp::Ordering::Greater,
            std::cmp::Ordering::Equal => self.length.cmp(&other.length),
            std::cmp::Ordering::Greater => std::cmp::Ordering::Less,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct SingleCacheKey((NodeIndex, KeySet));

pub(crate) struct CaveGraph {
    inner: StableGraph<Tile, u8, petgraph::Undirected>,
    start: NodeIndex,
    map: Map,
    raw_to_nidx: Vec<NodeIndex>,
}

impl CaveGraph {
    pub(crate) fn start(&self) -> ExploreState {
        ExploreState {
            pos: self.start,
            length: 0,
            keys: KeySet::new(),
        }
    }
    pub(crate) fn explored_map(&self, explore: ExploreState) -> Map {
        let (idx, _) = self
            .raw_to_nidx
            .iter()
            .enumerate()
            .find(|(_, n)| **n == explore.pos)
            .unwrap();
        println!("EXPLORE MAP: {:?}", explore);
        let mut m = self.map.clone();
        for t in &mut m.data {
            match t {
                Tile::Start => *t = Tile::Space,
                Tile::Door(k) => {
                    if explore.keys.contains(*k) {
                        *t = Tile::Space
                    }
                }
                Tile::Key(k) => {
                    if explore.keys.contains(*k) {
                        *t = Tile::Space
                    }
                }
                _ => {}
            }
        }
        m.data[idx] = Tile::Start;
        m
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
            map: m,
            start: start.expect("graph must have a start tile"),
            raw_to_nidx: node_idx,
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

    pub(crate) fn shortest_path(&self) -> u32 {
        let log = slog_scope::logger();
        //let mut queue = ExploreMinHeap::default();
        let mut queue = ExploreMinHeap::default();
        queue.insert(self.start());
        self.process_queue(&mut queue).unwrap()
    }

    fn process_queue<Q: ExploreQueue>(&self, queue: &mut Q) -> Option<u32> {
        let log = slog_scope::logger();

        let mut shortest_distance = None;
        let mut seen_explore = HashMap::new();

        let all_keys = self
            .inner
            .node_references()
            .filter_map(|(_, t)| match t {
                Tile::Key(k) => Some(*k),
                _ => None,
            })
            .fold(KeySet::new(), |acc, k| acc.insert(k));

        info!(log, "Looking for all keys in {:?}", all_keys);

        while let Some(explore) = queue.pop() {
            match seen_explore.entry(explore.cache_key()) {
                std::collections::hash_map::Entry::Occupied(mut o) => {
                    let prev = o.get_mut();
                    if *prev <= explore.length {
                        continue;
                    } else {
                        *prev = explore.length;
                    }
                }
                std::collections::hash_map::Entry::Vacant(mut v) => {
                    v.insert(explore.length);
                }
            }
            if let Some(shortest) = shortest_distance {
                if shortest <= explore.length {
                    continue;
                }
            }
            debug!(log, "TOTAL: {} Current {:?}", queue.len(), explore);
            trace!(log, "\n{}", self.explored_map(explore));
            let seen = explore.keys;
            let mut stack = Vec::new();

            let m = petgraph::algo::dijkstra(&self.inner, explore.pos, None, |e| {
                let src = e.source();
                let dst = e.target();
                self.edge_cost(src, dst, seen, &mut stack)
            });
            stack
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
                        if e.length <= *prev {
                            *prev = e.length;
                            info!(log, "Shortish Path {:?}", e);
                        }
                        false
                    } else {
                        true
                    }
                })
                .for_each(|e| {
                    queue.insert(e);
                });

            //debug!(log, "Dijkstra {:?}", m);
            // queue.extend(next_keys);

        }
        info!(log, "Shortest Path {:?}", shortest_distance);

        shortest_distance
    }

    // pub(crate) fn dijkstra(&self, explore: ExploreState) -> u32 {
    //     let log = slog_scope::logger();
    //     let all_keys = self
    //         .inner
    //         .node_references()
    //         .filter_map(|(_, t)| match t {
    //             Tile::Key(k) => Some(*k),
    //             _ => None,
    //         })
    //         .fold(KeySet::new(), |acc, k| acc.insert(k));

    //     info!(log, "Looking for all keys in {:?}", all_keys);
    //     let mut shortest_distance = None;

    //     let mut queue = std::collections::BinaryHeap::new();
    //     queue.push(std::cmp::Reverse(explore));
    //     let mut seen_explore = HashMap::new();

    //     info!(log, "Shortest Path {:?}", shortest_distance);
    //     return shortest_distance.unwrap();
    // }

    fn edge_cost(&self, src: NodeIndex, dst: NodeIndex, seen: KeySet, stack: &mut Vec<(NodeIndex, Key)>) -> PathCost {
        let src_t = self.inner.node_weight(src).unwrap();
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
        PathCost::Path(1)
    }
}
