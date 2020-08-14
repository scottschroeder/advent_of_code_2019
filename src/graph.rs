pub mod traverse {
    use petgraph::algo::Measure;
    use petgraph::visit::{EdgeRef, IntoEdges, VisitMap, Visitable};
    use scored::MinScored;
    use std::collections::hash_map::Entry::{Occupied, Vacant};
    use std::collections::{BinaryHeap, HashMap};
    use std::hash::Hash;

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum EdgeControl<T> {
        Continue(T),
        Break(T),
        Block,
    }

    pub fn dijkstra<G, F, K>(graph: G, start: G::NodeId, mut edge_cost: F) -> HashMap<G::NodeId, K>
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
