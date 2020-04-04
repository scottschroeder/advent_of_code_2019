use crate::orbital_data::transitive151::TransitiveClosure;
use petgraph::graphmap::GraphMap;

mod transitive151 {
    use fixedbitset::FixedBitSet;
    use petgraph::visit::*;
    use std::fmt::{Debug, Error, Formatter};

    pub struct TransitiveClosure {
        inner: FixedBitSet,
        width: usize,
    }

    impl Debug for TransitiveClosure {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
            write!(f, "TransitiveClosure {{")?;
            for src in 0..self.width {
                write!(f, "\n\t")?;
                for dst in 0..self.width {
                    write!(f, "{}", if self.is_reachable(src, dst) { 1 } else { 0 })?;
                }
            }
            write!(f, "\n}}")?;
            Ok(())
        }
    }

    impl TransitiveClosure {
        pub fn new(set: FixedBitSet, size: usize) -> TransitiveClosure {
            let mut tc = TransitiveClosure {
                inner: set,
                width: size,
            };
            tc
        }

        #[inline]
        pub fn is_reachable(&self, src: usize, dst: usize) -> bool {
            self.inner[self.index(src, dst)]
        }

        #[inline]
        fn index(&self, src: usize, dst: usize) -> usize {
            src * self.width + dst
        }

        pub fn connections(&self) -> usize {
            self.inner.ones().count()
        }
    }


    pub fn transitive_closure<G>(g: G) -> TransitiveClosure
        where G: NodeIndexable + NodeCount + IntoNeighbors + IntoNodeIdentifiers + Visitable
    {

        let n = g.node_count();
        let mut matrix = FixedBitSet::with_capacity(n * n);
        let mut dfs = Dfs::empty(g);

        for node in g.node_identifiers() {
            dfs.reset(g);
            dfs.move_to(node);
            let i = g.to_index(node);
            matrix.put(i * n + i);
            while let Some(visited) = dfs.next(g) {
                let i = i * n + g.to_index(visited);
                matrix.put(i);
            }
        }

        for idx in 0..n {
            matrix.set(idx * n + idx, false);
        }

        TransitiveClosure::new(matrix, n)
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
        self::transitive151::transitive_closure(&self.inner)
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
