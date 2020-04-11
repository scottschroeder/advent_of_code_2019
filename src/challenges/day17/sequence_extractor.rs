use blackout::BlackoutSeq;
// fn divide3<T, U>(seq: U) -> Divide3<T>
// where
//     T: PartialEq,
//     U: AsRef<[T]>,
// {
//     Divide3 {
//         a: vec![],
//         b: vec![],
//         c: vec![],
//         master: vec![],
//     }
// }

enum SubSeq3 {
    A,
    B,
    C,
}

#[derive(Debug, Clone, PartialEq)]
struct Divide3<'a, T> {
    a: &'a [T],
    b: &'a [T],
    c: &'a [T],
    //master: Vec<SubSeq3>,
}


fn divide_segments<T: PartialEq>(data: &[T], start: usize, end: usize) -> Option<Divide3<'_, T>> {
    if start + end > data.len() {
        return None;
    }
    let a = &data[..start];
    let c_start = data.len() - end;
    let c = &data[c_start..];
    let mut blk = BlackoutSeq::new(data);
    blk.blackout_seq(a);
    blk.blackout_seq(c);

    if let Some(b_longest) = blk.shortest() {
        for b_len in 1..b_longest.len()+1 {
            let mut b_test_blk = blk.clone();
            let b = &b_longest[..b_len];
            b_test_blk.blackout_seq(b);
            if b_test_blk.iter().count() == 0 {
                return Some(Divide3 { a, b, c })
            }
        }
    } else {
        return Some(Divide3 { a, b: &[], c })
    }
    None
}


#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn divide_segments_simple() {
        let data = "lrlrabstart".as_bytes();
        let d3 = divide_segments(data, 2, 5).unwrap();
        assert_eq!(d3, Divide3{a: "lr".as_bytes(), b: "ab".as_bytes(), c: "start".as_bytes()})
    }
}

mod blackout {

    fn find_first<T: PartialEq>(needle: &[T], haystack: &[T]) -> Option<usize> {
        if needle.len() > haystack.len() {
            return None;
        }
        let max_start = haystack.len() - needle.len();

        for idx in 0..max_start + 1 {
            if haystack[idx..].starts_with(needle) {
                return Some(idx);
            }
        }
        None
    }

    fn search<'a, T: PartialEq>(
        needle: &'a [T],
        haystack: &'a [T],
    ) -> NonOverlappingMatches<'a, T> {
        NonOverlappingMatches {
            needle,
            haystack,
            start_idx: 0,
        }
    }

    struct NonOverlappingMatches<'a, T> {
        needle: &'a [T],
        haystack: &'a [T],
        start_idx: usize,
    }

    impl<'a, T: PartialEq> Iterator for NonOverlappingMatches<'a, T> {
        type Item = usize;
        fn next(&mut self) -> Option<Self::Item> {
            if self.start_idx + self.needle.len() > self.haystack.len() {
                return None;
            }
            if self.needle.len() == 0 {
                return None
            }
            if let Some(m) = find_first(self.needle, &self.haystack[self.start_idx..]) {
                let loc = self.start_idx + m;
                self.start_idx = loc + self.needle.len();
                Some(loc)
            } else {
                self.start_idx = self.haystack.len();
                None
            }
        }
    }

    // TODO track the whitelisted areas?
    pub struct BlackoutSeq<'a, T> {
        inner: &'a [T],
        blackout: Vec<(usize, usize)>,
    }

    impl<'a, T> Clone for BlackoutSeq<'a, T> {
        fn clone(&self) -> Self {
            BlackoutSeq {
                inner: self.inner,
                blackout: self.blackout.clone(),
            }
        }
    }

    impl<'a, T> BlackoutSeq<'a, T> {
        pub fn new(seq: &'a [T]) -> BlackoutSeq<'a, T> {
            BlackoutSeq {
                inner: seq,
                blackout: vec![],
            }
        }

        pub fn shortest(&self) -> Option<&'a [T]> {
            let mut size = None;
            let mut start = 0;
            let mut update = |len, idx| {
                if len == 0 {
                    return
                }
                let min_len = size.get_or_insert(len);
                if len <= *min_len {
                    *min_len = len;
                    start = idx;
                }
            };

            let mut idx = 0usize;
            for (b_start, b_len) in &self.blackout {
                let s_len = b_start - idx;
                update(s_len, idx);
                idx = b_start + b_len;
            }
            update(self.inner.len() - idx, idx);
            size.map(|l|{&self.inner[start..start+l]})
        }

        fn blackout(&mut self, start: usize, len: usize) {
            let idx = match self
                .blackout
                .binary_search_by(|&(probe, _)| probe.cmp(&start))
            {
                Ok(x) => x,
                Err(x) => x,
            };
            self.blackout.insert(idx, (start, len))
        }

        pub fn iter(&self) -> BlackoutScanner<'_, T> {
            BlackoutScanner {
                inner: &self,
                idx: 0,
                blk: 0,
            }
        }
    }

    impl<'a, T: PartialEq> BlackoutSeq<'a, T> {
        pub fn blackout_seq(&mut self, seq: &[T]) {
            let new_blackouts = self.iter()
            .flat_map(|(idx, sub)| {
                search(seq, sub).map(move|m| idx + m)
            })
            .collect::<Vec<usize>>();
            for b in new_blackouts {
                self.blackout(b, seq.len());

            }
        }
    }


    pub struct BlackoutScanner<'a, T> {
        inner: &'a BlackoutSeq<'a, T>,
        idx: usize,
        blk: usize,
    }

    impl<'a, T> Iterator for BlackoutScanner<'a, T> {
        type Item = (usize, &'a [T]);
        fn next(&mut self) -> Option<Self::Item> {
            let seq = self.inner.inner;
            loop {
                if self.idx >= seq.len() {
                    return None;
                }
                let start = self.idx;
                let end = if let Some(&(blk_start, blk_len)) = self.inner.blackout.get(self.blk) {
                    self.blk += 1;
                    self.idx = blk_start + blk_len;
                    blk_start
                } else {
                    self.idx = seq.len();
                    seq.len()
                };
                if end > start {
                    return Some((start, &seq[start..end]));
                }
            }
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn blackout_none() {
            let seq = "hello world".as_bytes();
            let bs = BlackoutSeq::new(seq);
            let x = bs.iter().collect::<Vec<_>>();
            assert_eq!(x, vec![(0, "hello world".as_bytes())])
        }

        #[test]
        fn blackout_middle() {
            let seq = "hello world".as_bytes();
            let mut bs = BlackoutSeq::new(seq);
            bs.blackout(3, 2);
            let x = bs.iter().collect::<Vec<_>>();
            assert_eq!(x, vec![(0, "hel".as_bytes()), (5, " world".as_bytes()),])
        }

        #[test]
        fn blackout_beginning() {
            let seq = "hello world".as_bytes();
            let mut bs = BlackoutSeq::new(seq);
            bs.blackout(0, 2);
            let x = bs.iter().collect::<Vec<_>>();
            assert_eq!(x, vec![(2, "llo world".as_bytes()),])
        }
        #[test]
        fn blackout_end() {
            let seq = "hello world".as_bytes();
            let mut bs = BlackoutSeq::new(seq);
            bs.blackout(8, 3);
            let x = bs.iter().collect::<Vec<_>>();
            assert_eq!(x, vec![(0, "hello wo".as_bytes()),])
        }

        #[test]
        fn blackout_two() {
            let seq = "hello world".as_bytes();
            let mut bs = BlackoutSeq::new(seq);
            bs.blackout(2, 3);
            bs.blackout(8, 1);
            let x = bs.iter().collect::<Vec<_>>();
            assert_eq!(
                x,
                vec![
                    (0, "he".as_bytes()),
                    (5, " wo".as_bytes()),
                    (9, "ld".as_bytes()),
                ]
            )
        }

        #[test]
        fn find_not_there() {
            assert_eq!(find_first("foo".as_bytes(), "fizzbuzz".as_bytes()), None,)
        }

        #[test]
        fn find_too_small() {
            assert_eq!(find_first("foobar".as_bytes(), "fizz".as_bytes()), None,)
        }

        #[test]
        fn find_beginning() {
            assert_eq!(find_first("foo".as_bytes(), "foobar".as_bytes()), Some(0),)
        }
        #[test]
        fn find_end() {
            assert_eq!(find_first("bar".as_bytes(), "foobar".as_bytes()), Some(3),)
        }
        #[test]
        fn find_middle() {
            assert_eq!(find_first("oob".as_bytes(), "foobar".as_bytes()), Some(1),)
        }
        #[test]
        fn find_empty() {
            assert_eq!(find_first("".as_bytes(), "foobar".as_bytes()), Some(0),)
        }

        #[test]
        fn search_missing() {
            let actual = search("foo".as_bytes(), "fizzbuzz".as_bytes()).collect::<Vec<_>>();
            assert_eq!(actual, vec![])
        }

        #[test]
        fn search_beginning() {
            let actual = search("foo".as_bytes(), "foobar".as_bytes()).collect::<Vec<_>>();
            assert_eq!(actual, vec![0])
        }
        #[test]
        fn search_end() {
            let actual = search("bar".as_bytes(), "foobar".as_bytes()).collect::<Vec<_>>();
            assert_eq!(actual, vec![3])
        }
        #[test]
        fn search_too_long() {
            let actual = search("fizzbuzz".as_bytes(), "foobar".as_bytes()).collect::<Vec<_>>();
            assert_eq!(actual, vec![])
        }
        #[test]
        fn search_empty() {
            let actual = search("".as_bytes(), "foobar".as_bytes()).collect::<Vec<_>>();
            assert_eq!(actual, vec![])
        }

        #[test]
        fn search_two() {
            let actual = search("foo".as_bytes(), "theres foobar and foobaz".as_bytes()).collect::<Vec<_>>();
            assert_eq!(actual, vec![7, 18])
        }
        #[test]
        fn double_blackout() {
            let data = "foobar foobaz fizzbuzz and fizzle".as_bytes();
            let mut b = BlackoutSeq::new(data);
            b.blackout_seq("foo".as_bytes());
            b.blackout_seq("fizz".as_bytes());
            let actual = b.iter().map(|(_, s)| s).collect::<Vec<_>>();
            assert_eq!(actual, vec![
                "bar ".as_bytes(),
                "baz ".as_bytes(),
                "buzz and ".as_bytes(),
                "le".as_bytes(),
            ])
        }
        #[test]
        fn blackout_shortest_start() {
            let data = "wargXXXfizzpopXXXwargwarg".as_bytes();
            let mut b = BlackoutSeq::new(data);
            b.blackout_seq("XXX".as_bytes());
            assert_eq!(b.shortest(), Some("warg".as_bytes()));
        }
        #[test]
        fn blackout_shortest_end() {
            let data = "wargwargXXXfizzpopXXXwarg".as_bytes();
            let mut b = BlackoutSeq::new(data);
            b.blackout_seq("XXX".as_bytes());
            assert_eq!(b.shortest(), Some("warg".as_bytes()));
        }
        #[test]
        fn blackout_shortest_middle() {
            let data = "lemonXXXwargXXXfizzpopXXXwargwarg".as_bytes();
            let mut b = BlackoutSeq::new(data);
            b.blackout_seq("XXX".as_bytes());
            assert_eq!(b.shortest(), Some("warg".as_bytes()));
        }
        #[test]
        fn blackout_shortest_start_with_blackout() {
            let data = "XXXwarg".as_bytes();
            let mut b = BlackoutSeq::new(data);
            b.blackout_seq("XXX".as_bytes());
            assert_eq!(b.shortest(), Some("warg".as_bytes()));
        }
    }
}
