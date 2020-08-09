use super::map_reader::Map;
use super::{KeySet, Tile};
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;

use petgraph::stable_graph::{NodeIndex, StableGraph};
use petgraph::visit::EdgeRef;
use crate::graph::traverse::{EdgeControl, dijkstra};

pub(crate) trait ExploreState:
    fmt::Debug + Clone + PartialEq + PartialOrd + Ord + IntoIterator<Item = (usize, NodeIndex)>
{
    type CacheKey: Copy + PartialEq + Eq + Hash;
    fn length(&self) -> u32;
    fn keys(&self) -> KeySet;
    fn update(&self, idx: usize, pos: NodeIndex, length: u32, keys: KeySet) -> Self;
    fn cache_key(&self) -> Self::CacheKey;
}

impl ExploreState for SingleState {
    type CacheKey = (NodeIndex, KeySet);
    #[inline]
    fn length(&self) -> u32 {
        self.length
    }
    #[inline]
    fn keys(&self) -> KeySet {
        self.keys
    }
    #[inline]
    fn cache_key(&self) -> Self::CacheKey {
        (self.pos, self.keys)
    }
    fn update(&self, _: usize, pos: NodeIndex, length: u32, keys: KeySet) -> Self {
        SingleState {
            pos,
            length: self.length + length,
            keys,
        }
    }
}

impl IntoIterator for SingleState {
    type Item = (usize, NodeIndex);
    type IntoIter = std::iter::Once<(usize, NodeIndex)>;
    fn into_iter(self) -> Self::IntoIter {
        std::iter::once((0, self.pos))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct SingleState {
    pub(crate) pos: NodeIndex,
    pub(crate) length: u32,
    pub(crate) keys: KeySet,
}

type QuadNodeIndex = [NodeIndex; 4];

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct QuadState {
    pub(crate) pos: QuadNodeIndex,
    pub(crate) length: u32,
    pub(crate) keys: KeySet,
}

impl ExploreState for QuadState {
    type CacheKey = (QuadNodeIndex, KeySet);
    #[inline]
    fn length(&self) -> u32 {
        self.length
    }
    #[inline]
    fn keys(&self) -> KeySet {
        self.keys
    }
    #[inline]
    fn cache_key(&self) -> Self::CacheKey {
        (self.pos, self.keys)
    }
    fn update(&self, idx: usize, pos: NodeIndex, length: u32, keys: KeySet) -> Self {
        let mut new_nodes = self.pos.clone();
        new_nodes[idx] = pos;
        QuadState {
            pos: new_nodes,
            length: self.length + length,
            keys,
        }
    }
}

pub(crate) struct QuadIter {
    inner: QuadNodeIndex,
    idx: usize,
}

impl Iterator for QuadIter {
    type Item = (usize, NodeIndex);
    fn next(&mut self) -> Option<Self::Item> {
        let mut ret = None;
        if self.idx < 4 {
            ret = Some((self.idx, self.inner[self.idx]));
            self.idx += 1;
        }
        return ret;
    }
}

impl IntoIterator for QuadState {
    type Item = (usize, NodeIndex);
    type IntoIter = QuadIter;
    fn into_iter(self) -> Self::IntoIter {
        QuadIter {
            inner: self.pos.clone(),
            idx: 0,
        }
    }
}

impl PartialOrd for SingleState {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for SingleState {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .length
            .cmp(&self.length)
            .then(self.keys.len().cmp(&other.keys.len()))
    }
}

impl PartialOrd for QuadState {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for QuadState {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .length
            .cmp(&self.length)
            .then(self.keys.len().cmp(&other.keys.len()))
    }
}

pub(crate) struct CaveGraph {
    inner: StableGraph<Tile, u32, petgraph::Undirected>,
    all_keys: KeySet,
    map: Map,
    raw_to_nidx: Vec<NodeIndex>,
    nidx_to_tile: Vec<Tile>,
}

impl CaveGraph {
    fn compress(&mut self) {
        let mut g = self.inner.clone();
        g.clear_edges();
        for (nidx, tile) in self.nodes() {
            match tile {
                Tile::Start | Tile::Key(_) | Tile::Door(_) => {
                    let m = dijkstra(&self.inner, nidx, |e| {
                        let dst: NodeIndex = e.target();
                        let dst_t = self.nidx_to_tile[dst.index()];
                        let w = *e.weight();
                        match dst_t {
                            Tile::Wall => EdgeControl::Block,
                            Tile::Space | Tile::Start => EdgeControl::Continue(w),
                            Tile::Door(_) => EdgeControl::Break(w),
                            Tile::Key(_) => EdgeControl::Break(w),
                        }
                    });
                    for (dst, w) in m {
                        if nidx >= dst {
                            continue;
                        }
                        let dst_t: Tile = self.nidx_to_tile[dst.index()];
                        match dst_t {
                            Tile::Start | Tile::Door(_) | Tile::Key(_) => {
                                g.add_edge(nidx, dst, w);
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
        g.retain_nodes(|g, n| match *g.node_weight(n).unwrap() {
            Tile::Wall | Tile::Space => false,
            _ => true,
        });
        self.inner = g;
    }
    pub(crate) fn explored_map<S: ExploreState>(&self, explore: S) -> Map {
        let mut m = self.map.clone();
        for t in &mut m.data {
            match t {
                Tile::Start => *t = Tile::Space,
                Tile::Door(k) => {
                    if explore.keys().contains(*k) {
                        *t = Tile::Space
                    }
                }
                Tile::Key(k) => {
                    if explore.keys().contains(*k) {
                        *t = Tile::Space
                    }
                }
                _ => {}
            }
        }
        for idx in explore.into_iter().map(|(_, n)| n) {
            let (idx, _) = self
                .raw_to_nidx
                .iter()
                .enumerate()
                .find(|(_, n)| **n == idx)
                .unwrap();
            m.data[idx] = Tile::Start;
        }
        m
    }
    pub(crate) fn from_map(m: Map) -> Self {
        let mut node_idx = Vec::with_capacity(m.data.len());
        let mut g = StableGraph::default();
        let mut max_nidx = 0;
        let mut all_keys = KeySet::new();
        for t in &m.data {
            let idx = g.add_node(*t);
            max_nidx = std::cmp::max(max_nidx, idx.index());
            node_idx.push(idx);
            match t {
                Tile::Key(k) => all_keys = all_keys.insert(*k),
                _ => {}
            }
        }

        let mut nidx_to_tile = vec![Tile::Space; max_nidx];
        for nidx in 0..max_nidx {
            if let Some(t) = g.node_weight(NodeIndex::new(nidx)) {
                nidx_to_tile[nidx] = *t;
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
            all_keys,
            raw_to_nidx: node_idx,
            nidx_to_tile,
        };
        cg.compress();
        cg
    }

    pub(crate) fn dot(&self) -> String {
        format!("{:?}", petgraph::dot::Dot::new(&self.inner))
    }

    fn nodes(&self) -> impl Iterator<Item = (NodeIndex, Tile)> + '_ {
        self.inner
            .node_indices()
            .map(move |idx| (idx, *self.inner.node_weight(idx).unwrap()))
    }

    pub(crate) fn start(&self) -> impl Iterator<Item = NodeIndex> + '_ {
        self.nodes()
            .filter_map(|(idx, t)| if t == Tile::Start { Some(idx) } else { None })
    }

    pub(crate) fn shortest_path<S: ExploreState>(&self, start: S) -> Option<u32> {
        let mut shortest_distance = None;
        let mut distances = HashMap::new();
        let mut priority_queue = std::collections::BinaryHeap::new();

        log::info!("Looking for all keys in {:?}", self.all_keys);

        priority_queue.push(start);

        while let Some(explore) = priority_queue.pop() {
            match distances.entry(explore.cache_key()) {
                std::collections::hash_map::Entry::Occupied(mut o) => {
                    let prev = o.get_mut();
                    if *prev <= explore.length() {
                        continue;
                    } else {
                        *prev = explore.length();
                    }
                }
                std::collections::hash_map::Entry::Vacant(v) => {
                    v.insert(explore.length());
                }
            }
            if let Some(shortest) = shortest_distance {
                if shortest <= explore.length() {
                    continue;
                }
            }
            log::debug!("TOTAL: {} Current {:?}", priority_queue.len(), explore);
            log::trace!("\n{}", self.explored_map(explore.clone()));
            let seen = explore.keys();

            explore
                .clone()
                .into_iter()
                .flat_map(|(state_idx, pos)| {
                    dijkstra(&self.inner, pos, |e| {
                        let dst = e.target();
                        self.edge_cost(dst, seen, *e.weight())
                    })
                    .into_iter()
                    .map(move |(k, v)| (state_idx, k, v))
                })
                .filter_map(|(state_idx, n, c)| {
                    let t: Tile = self.nidx_to_tile[n.index()];
                    if let Tile::Key(k) = t {
                        let keys = seen.insert(k);
                        if keys != seen {
                            Some(explore.update(state_idx, n, c, keys))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .filter(|e| {
                    if e.keys() == self.all_keys {
                        let prev = shortest_distance.get_or_insert(e.length());
                        if e.length() <= *prev {
                            *prev = e.length();
                            log::info!("Shortish Path {:?}", e);
                        }
                        false
                    } else {
                        true
                    }
                })
                .for_each(|e| {
                    priority_queue.push(e);
                });
        }
        log::info!("Shortest Path {:?}", shortest_distance);

        shortest_distance
    }

    fn edge_cost(&self, dst: NodeIndex, seen: KeySet, weight: u32) -> EdgeControl<u32> {
        //let dst_t2 = self.inner.node_weight(dst).unwrap();
        let dst_t = self.nidx_to_tile[dst.index()];
        match dst_t {
            Tile::Door(k) => {
                if !seen.contains(k) {
                    EdgeControl::Block
                } else {
                    EdgeControl::Continue(weight)
                }
            }
            Tile::Key(k) => {
                if !seen.contains(k) {
                    EdgeControl::Break(weight)
                } else {
                    EdgeControl::Continue(weight)
                }
            }
            // Other things are unreachable
            _ => EdgeControl::Continue(weight),
        }
    }
}
