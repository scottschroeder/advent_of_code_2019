use crate::orbital_data::warshall::TransitiveClosure;
use petgraph::graphmap::GraphMap;

mod warshall {
    use petgraph::visit::*;
    use std::fmt::{Debug, Error, Formatter};

    pub struct TransitiveClosure {
        inner: Vec<bool>,
        size: usize,
    }

    impl Debug for TransitiveClosure {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
            write!(f, "TransitiveClosure {{")?;
            for src in 0..self.size {
                write!(f, "\n\t")?;
                for dst in 0..self.size {
                    write!(f, "{}", if self.is_reachable(src, dst) { 1 } else { 0 })?;
                }
            }
            write!(f, "\n}}")?;
            Ok(())
        }
    }

    impl TransitiveClosure {
        pub fn new(size: usize) -> TransitiveClosure {
            TransitiveClosure {
                inner: vec![false; size * size],
                size,
            }
        }

        #[inline]
        pub fn is_reachable(&self, src: usize, dst: usize) -> bool {
            self.inner[src * self.size + dst]
        }

        #[inline]
        fn connect(&mut self, src: usize, dst: usize) {
            self.inner[src * self.size + dst] = true
        }

        pub fn connections(&self) -> usize {
            self.inner.iter().map(|b| if *b { 1 } else { 0 }).sum()
        }
    }

    pub(crate) fn warshall_transitive_closure<G>(g: G) -> TransitiveClosure
    where
        G: GraphBase + GetAdjacencyMatrix + IntoNodeIdentifiers + NodeIndexable,
    {
        let size = g.node_bound() as usize;
        let mut tc = TransitiveClosure::new(size);

        let adj = g.adjacency_matrix();

        for i_node in g.node_identifiers() {
            let idx = g.to_index(i_node);
            for j_node in g.node_identifiers() {
                let jdx = g.to_index(j_node);
                if g.is_adjacent(&adj, i_node, j_node) {
                    tc.connect(idx, jdx)
                }
            }
        }

        for k in 0..tc.size {
            for i in 0..tc.size {
                for j in 0..tc.size {
                    if tc.is_reachable(i, k) && tc.is_reachable(k, j) {
                        tc.connect(i, j)
                    }
                }
            }
        }

        tc
    }
}

#[derive(Debug, Clone, Default)]
pub struct OrbitalMap<'a> {
    inner: GraphMap<&'a str, (), petgraph::Directed>,
}

impl<'a> OrbitalMap<'a> {
    fn add_edge(&mut self, src: &'a str, dst: &'a str) {
        self.inner.add_edge(src, dst, ());
    }
    pub fn from_orbital_data(input: &'a str) -> OrbitalMap<'a> {
        let mut new = OrbitalMap::default();
        for (src, dst) in parse_orbital_data(input) {
            new.add_edge(src, dst);
        }
        new
    }
    pub fn transitive_closure(&self) -> TransitiveClosure {
        self::warshall::warshall_transitive_closure(&self.inner)
    }

    pub fn shortest_path(&self, src: &str, dst: &str) -> Option<usize> {
        let undirected: GraphMap<&str, (), petgraph::Undirected> =
            GraphMap::from_edges(self.inner.all_edges());
        let x = petgraph::algo::dijkstra(&undirected, src, Some(dst), |(s, d, _)| {
            //trace!(slog_scope::logger(), "edge {} -> {}", s, d);
            1
        });
        x.get("SAN").and_then(|x| {
            if *x >= 2 {
                Some(x - 2 as usize)
            } else {
                trace!(slog_scope::logger(), "Got too short of distance: {:#?}", x);
                None
            }
        })
    }
}
pub fn parse_orbital_data(input: &str) -> Vec<(&str, &str)> {
    input
        .lines()
        .flat_map(|l| l.split(')'))
        .collect::<Vec<&str>>()
        .chunks(2)
        .map(|s| (s[0], s[1]))
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_orbital_check() {
        assert_eq!(parse_orbital_data("a)b"), vec![("a", "b")],);
        assert_eq!(
            parse_orbital_data("a)b\nb)c\na)d"),
            vec![("a", "b"), ("b", "c"), ("a", "d"),],
        );
    }
}
