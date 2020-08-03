use super::map_reader::Map;
use super::{Key, KeySet, Tile};
use std::collections::HashMap;

use petgraph::stable_graph::{NodeIndex, StableGraph};
use petgraph::visit::{EdgeRef, IntoNodeReferences};
use std::ops::Add;
use traverse::{dijkstra, EdgeControl};

trait ExploreQueue: Default {
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
    all_keys: KeySet,
    map: Map,
    raw_to_nidx: Vec<NodeIndex>,
    nidx_to_tile: Vec<Tile>,
}

impl CaveGraph {
    pub(crate) fn start(&self) -> ExploreState {
        ExploreState {
            pos: self.start,
            length: 0,
            keys: KeySet::new(),
        }
    }

    pub(crate) fn compress(&self) -> CaveGraph {
        let mut g = StableGraph::default();
        for nidx in self.inner.node_indices() {
            let tile = self.inner.node_weight(nidx).unwrap();
            match tile {
                Tile::Start | Tile::Key(_) => {
                    let m = traverse::dijkstra(&self.inner, nidx, |e| {
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
                        let dst_t = self.nidx_to_tile[dst.index()];
                        //g.add_edge(a, b, weight)
                    }
                }
                _ => {}
            }


        }


        CaveGraph {
            inner: g,
            start: self.start,
            all_keys: self.all_keys,
            map: self.map.clone(),
            raw_to_nidx: vec![], // TODO
            nidx_to_tile: vec![], // TODO
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
        let mut max_nidx = 0;
        let mut all_keys = KeySet::new();
        for t in &m.data {
            let idx = g.add_node(*t);
            max_nidx = std::cmp::max(max_nidx, idx.index());
            node_idx.push(idx);
            match t {
                Tile::Start => start = Some(idx),
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
            start: start.expect("graph must have a start tile"),
            all_keys,
            raw_to_nidx: node_idx,
            nidx_to_tile,
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
        //let mut queue = ExploreMinHeap::default();
        let mut queue = ExploreMinHeap::default();
        queue.insert(self.start());
        self.process_queue(&mut queue).unwrap()
    }

    fn process_queue<Q: ExploreQueue>(&self, queue: &mut Q) -> Option<u32> {
        let mut shortest_distance = None;
        let mut seen_explore = HashMap::new();

        log::info!("Looking for all keys in {:?}", self.all_keys);

        let mut stack = Vec::new();
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
            log::debug!("TOTAL: {} Current {:?}", queue.len(), explore);
            log::trace!("\n{}", self.explored_map(explore));
            let seen = explore.keys;

            stack.truncate(0);

            let m = traverse::dijkstra(&self.inner, explore.pos, |e| {
                let dst = e.target();
                self.edge_cost(dst, seen, &mut stack)
            });

            stack
                .iter()
                .filter_map(|(n, k)| {
                    m.get(&n).map(|c| ExploreState {
                        pos: *n,
                        length: c + explore.length,
                        keys: seen.insert(*k),
                    })
                })
                .filter(|e| {
                    if e.keys == self.all_keys {
                        let prev = shortest_distance.get_or_insert(e.length);
                        if e.length <= *prev {
                            *prev = e.length;
                            log::info!("Shortish Path {:?}", e);
                        }
                        false
                    } else {
                        true
                    }
                })
                .for_each(|e| {
                    queue.insert(e);
                });

            //log::debug!( "Dijkstra {:?}", m);
            // queue.extend(next_keys);
        }
        log::info!("Shortest Path {:?}", shortest_distance);

        shortest_distance
    }

    fn edge_cost(
        &self,
        dst: NodeIndex,
        seen: KeySet,
        stack: &mut Vec<(NodeIndex, Key)>,
    ) -> EdgeControl<u32> {
        //let dst_t2 = self.inner.node_weight(dst).unwrap();
        let dst_t = self.nidx_to_tile[dst.index()];
        match dst_t {
            Tile::Door(k) => {
                if !seen.contains(k) {
                    EdgeControl::Block
                } else {
                    EdgeControl::Continue(1)
                }
            }
            Tile::Key(k) => {
                if !seen.contains(k) {
                    stack.push((dst, k));
                    EdgeControl::Break(1)
                } else {
                    EdgeControl::Continue(1)
                }
            }
            // Other things are unreachable
            _ => EdgeControl::Continue(1),
        }
    }
}

mod traverse {
    use petgraph::algo::Measure;
    use petgraph::visit::{EdgeRef, IntoEdges, VisitMap, Visitable};
    use scored::MinScored;
    use std::collections::hash_map::Entry::{Occupied, Vacant};
    use std::collections::{BinaryHeap, HashMap};
    use std::hash::Hash;

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub(crate) enum EdgeControl<T> {
        Continue(T),
        Break(T),
        Block,
    }

    pub(crate) fn dijkstra<G, F, K>(
        graph: G,
        start: G::NodeId,
        mut edge_cost: F,
    ) -> HashMap<G::NodeId, K>
    where
        G: IntoEdges + Visitable,
        G::NodeId: Eq + Hash,
        F: FnMut(G::EdgeRef) -> EdgeControl<K>,
        K: Measure + Copy,
    {
        let mut visited = graph.visit_map();
        let mut scores = HashMap::new();
        let mut visit_next = BinaryHeap::new();
        let zero_score = K::default();
        scores.insert(start, zero_score);
        visit_next.push(MinScored(zero_score, start));
        while let Some(MinScored(node_score, node)) = visit_next.pop() {
            if visited.is_visited(&node) {
                continue;
            }
            // if goal.as_ref() == Some(&node) {
            //     break;
            // }
            for edge in graph.edges(node) {
                let next = edge.target();
                if visited.is_visited(&next) {
                    continue;
                }
                let this_edge = match edge_cost(edge) {
                    EdgeControl::Continue(c) => c,
                    EdgeControl::Break(c) => {
                        visited.visit(next);
                        c
                    }
                    EdgeControl::Block => continue,
                };
                let next_score = node_score + this_edge;
                match scores.entry(next) {
                    Occupied(ent) => {
                        if next_score < *ent.get() {
                            *ent.into_mut() = next_score;
                            visit_next.push(MinScored(next_score, next));
                        }
                    }
                    Vacant(ent) => {
                        ent.insert(next_score);
                        visit_next.push(MinScored(next_score, next));
                    }
                }
            }
            visited.visit(node);
        }
        scores
    }

    mod scored {
        use std::cmp::Ordering;

        /// `MinScored<K, T>` holds a score `K` and a scored object `T` in
        /// a pair for use with a `BinaryHeap`.
        ///
        /// `MinScored` compares in reverse order by the score, so that we can
        /// use `BinaryHeap` as a min-heap to extract the score-value pair with the
        /// least score.
        ///
        /// **Note:** `MinScored` implements a total order (`Ord`), so that it is
        /// possible to use float types as scores.
        #[derive(Copy, Clone, Debug)]
        pub struct MinScored<K, T>(pub K, pub T);

        impl<K: PartialOrd, T> PartialEq for MinScored<K, T> {
            #[inline]
            fn eq(&self, other: &MinScored<K, T>) -> bool {
                self.cmp(other) == Ordering::Equal
            }
        }

        impl<K: PartialOrd, T> Eq for MinScored<K, T> {}

        impl<K: PartialOrd, T> PartialOrd for MinScored<K, T> {
            #[inline]
            fn partial_cmp(&self, other: &MinScored<K, T>) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        impl<K: PartialOrd, T> Ord for MinScored<K, T> {
            #[inline]
            fn cmp(&self, other: &MinScored<K, T>) -> Ordering {
                let a = &self.0;
                let b = &other.0;
                if a == b {
                    Ordering::Equal
                } else if a < b {
                    Ordering::Greater
                } else if a > b {
                    Ordering::Less
                } else if a.ne(a) && b.ne(b) {
                    // these are the NaN cases
                    Ordering::Equal
                } else if a.ne(a) {
                    // Order NaN less, so that it is last in the MinScore order
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            }
        }
    }
}
